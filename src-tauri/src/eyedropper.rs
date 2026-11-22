//! Ekranın herhangi bir yerinden renk alan damlalık.
//!
//! Tarayıcının `EyeDropper` API'si WebView2'de hemen iptal oluyor; bu yüzden ekranın bir görüntüsü
//! alınıp tüm monitörleri kaplayan bir pencerede gösterilir ve renk o görüntüden okunur.
//! Tıklama/Enter seçer, Esc/sağ tık vazgeçer, ok tuşları imleci birer piksel kaydırır.

use std::ptr::{null, null_mut};
use std::sync::atomic::{AtomicBool, Ordering};

use windows_sys::Win32::Foundation::{HINSTANCE, HWND, LPARAM, LRESULT, POINT, RECT, WPARAM};
use windows_sys::Win32::Graphics::Gdi::{
    BeginPaint, BitBlt, CAPTUREBLT, COLORONCOLOR, CreateCompatibleBitmap, CreateCompatibleDC,
    CreateFontW, CreateSolidBrush, DT_CENTER, DT_SINGLELINE, DT_VCENTER, DeleteDC, DeleteObject,
    DrawTextW, EndPaint, FillRect, FrameRect, GetDC, GetMonitorInfoW, GetPixel, HBITMAP, HDC,
    HGDIOBJ, InvalidateRect, MONITOR_DEFAULTTONEAREST, MONITORINFO, MonitorFromPoint, PAINTSTRUCT,
    ReleaseDC, SRCCOPY, SelectObject, SetBkMode, SetStretchBltMode, SetTextColor, StretchBlt,
    TRANSPARENT, UpdateWindow,
};
use windows_sys::Win32::System::LibraryLoader::GetModuleHandleW;
use windows_sys::Win32::UI::HiDpi::{
    DPI_AWARENESS_CONTEXT_PER_MONITOR_AWARE_V2, GetDpiForMonitor, MDT_EFFECTIVE_DPI,
    SetThreadDpiAwarenessContext,
};
use windows_sys::Win32::UI::Input::KeyboardAndMouse::{
    VK_DOWN, VK_ESCAPE, VK_LEFT, VK_RETURN, VK_RIGHT, VK_UP,
};
use windows_sys::Win32::UI::WindowsAndMessaging::*;

/// Büyüteçte görünen piksel sayısı (tek sayı: ortadaki seçilir) ve 96 DPI'da bir pikselin boyu.
const CELLS: i32 = 13;
const CELL: i32 = 10;
const LABEL: i32 = 30;
const GAP: i32 = 20;

static OPEN: AtomicBool = AtomicBool::new(false);

struct Picker {
    origin: POINT,
    size: (i32, i32),
    /// Ekran görüntüsü ve titremesiz çizim için ara tampon.
    shot: HDC,
    back: HDC,
    cursor: POINT,
    lens: RECT,
    done: bool,
    result: Option<u32>,
}

/// Renk seçilene ya da vazgeçilene kadar bekler; `#rrggbb` döndürür.
pub fn pick() -> Option<String> {
    if OPEN.swap(true, Ordering::AcqRel) {
        return None;
    }
    let color = unsafe {
        // Fiziksel piksellerle çalış; karışık DPI'lı monitörlerde koordinatlar kaymasın.
        let dpi = SetThreadDpiAwarenessContext(DPI_AWARENESS_CONTEXT_PER_MONITOR_AWARE_V2);
        let color = run();
        SetThreadDpiAwarenessContext(dpi);
        color
    };
    OPEN.store(false, Ordering::Release);
    color.map(hex)
}

/// COLORREF (0x00BBGGRR) → `#rrggbb`.
fn hex(color: u32) -> String {
    format!(
        "#{:02x}{:02x}{:02x}",
        color & 0xFF,
        (color >> 8) & 0xFF,
        (color >> 16) & 0xFF
    )
}

const fn rgb(r: u32, g: u32, b: u32) -> u32 {
    r | (g << 8) | (b << 16)
}

fn wide(s: &str) -> Vec<u16> {
    s.encode_utf16().chain([0]).collect()
}

/// Ekran boyutunda bir bellek DC'si; `free` ile bırakılır.
unsafe fn buffer(screen: HDC, (w, h): (i32, i32)) -> (HDC, HBITMAP, HGDIOBJ) {
    unsafe {
        let dc = CreateCompatibleDC(screen);
        let bitmap = CreateCompatibleBitmap(screen, w, h);
        let old = SelectObject(dc, bitmap);
        (dc, bitmap, old)
    }
}

unsafe fn free((dc, bitmap, old): (HDC, HBITMAP, HGDIOBJ)) {
    unsafe {
        SelectObject(dc, old);
        DeleteObject(bitmap);
        DeleteDC(dc);
    }
}

unsafe fn run() -> Option<u32> {
    unsafe {
        let origin = POINT {
            x: GetSystemMetrics(SM_XVIRTUALSCREEN),
            y: GetSystemMetrics(SM_YVIRTUALSCREEN),
        };
        let size = (
            GetSystemMetrics(SM_CXVIRTUALSCREEN),
            GetSystemMetrics(SM_CYVIRTUALSCREEN),
        );
        let screen = GetDC(null_mut());
        let shot = buffer(screen, size);
        let back = buffer(screen, size);
        let captured = BitBlt(
            shot.0,
            0,
            0,
            size.0,
            size.1,
            screen,
            origin.x,
            origin.y,
            SRCCOPY | CAPTUREBLT,
        ) != 0;
        ReleaseDC(null_mut(), screen);

        let mut result = None;
        if captured {
            let mut picker = Picker {
                origin,
                size,
                shot: shot.0,
                back: back.0,
                cursor: POINT { x: 0, y: 0 },
                lens: RECT {
                    left: 0,
                    top: 0,
                    right: 0,
                    bottom: 0,
                },
                done: false,
                result: None,
            };
            show(&mut picker);
            result = picker.result;
        }
        free(back);
        free(shot);
        result
    }
}

unsafe fn show(picker: &mut Picker) {
    unsafe {
        let instance: HINSTANCE = GetModuleHandleW(null());
        let class = wide("BlazonEyedropper");
        let mut wc: WNDCLASSW = std::mem::zeroed();
        wc.lpfnWndProc = Some(window_proc);
        wc.hInstance = instance;
        wc.hCursor = LoadCursorW(null_mut(), IDC_CROSS);
        wc.lpszClassName = class.as_ptr();
        RegisterClassW(&wc); // ikinci açılışta zaten kayıtlıdır

        let state: *mut Picker = picker;
        let hwnd = CreateWindowExW(
            WS_EX_TOPMOST | WS_EX_TOOLWINDOW,
            class.as_ptr(),
            class.as_ptr(),
            WS_POPUP,
            (*state).origin.x,
            (*state).origin.y,
            (*state).size.0,
            (*state).size.1,
            null_mut(),
            null_mut(),
            instance,
            null(),
        );
        if hwnd.is_null() {
            return;
        }
        SetWindowLongPtrW(hwnd, GWLP_USERDATA, state as isize);
        track(hwnd, &mut *state);
        ShowWindow(hwnd, SW_SHOW);
        SetForegroundWindow(hwnd);
        UpdateWindow(hwnd);

        let mut msg: MSG = std::mem::zeroed();
        while !(*state).done && GetMessageW(&mut msg, null_mut(), 0, 0) > 0 {
            TranslateMessage(&msg);
            DispatchMessageW(&msg);
        }
        SetWindowLongPtrW(hwnd, GWLP_USERDATA, 0);
        DestroyWindow(hwnd);
    }
}

impl Picker {
    fn color(&self) -> u32 {
        unsafe { GetPixel(self.shot, self.cursor.x, self.cursor.y) & 0x00FF_FFFF }
    }

    fn finish(&mut self, result: Option<u32>) {
        if !self.done {
            self.done = true;
            self.result = result;
        }
    }

    /// Büyütecin yeri: imlecin sağ altı, monitör kenarına sığmazsa öbür yanı.
    fn place_lens(&self) -> RECT {
        unsafe {
            let at = POINT {
                x: self.cursor.x + self.origin.x,
                y: self.cursor.y + self.origin.y,
            };
            let monitor = MonitorFromPoint(at, MONITOR_DEFAULTTONEAREST);
            let mut info: MONITORINFO = std::mem::zeroed();
            info.cbSize = size_of::<MONITORINFO>() as u32;
            GetMonitorInfoW(monitor, &mut info);
            let (mut dpi, mut dpi_y) = (96u32, 96u32);
            GetDpiForMonitor(monitor, MDT_EFFECTIVE_DPI, &mut dpi, &mut dpi_y);
            lens_rect(self.cursor, to_local(info.rcMonitor, self.origin), dpi)
        }
    }
}

fn to_local(r: RECT, origin: POINT) -> RECT {
    RECT {
        left: r.left - origin.x,
        top: r.top - origin.y,
        right: r.right - origin.x,
        bottom: r.bottom - origin.y,
    }
}

fn lens_rect(cursor: POINT, monitor: RECT, dpi: u32) -> RECT {
    let scale = |v: i32| v * dpi as i32 / 96;
    let (side, gap) = (CELLS * scale(CELL), scale(GAP));
    let height = side + scale(LABEL);
    let fit = |at: i32, len: i32, end: i32| {
        if at + gap + len <= end {
            at + gap
        } else {
            at - gap - len
        }
    };
    let left = fit(cursor.x, side, monitor.right);
    let top = fit(cursor.y, height, monitor.bottom);
    RECT {
        left,
        top,
        right: left + side,
        bottom: top + height,
    }
}

unsafe fn track(hwnd: HWND, picker: &mut Picker) {
    unsafe {
        let mut at = POINT { x: 0, y: 0 };
        GetCursorPos(&mut at);
        picker.cursor = POINT {
            x: (at.x - picker.origin.x).clamp(0, picker.size.0 - 1),
            y: (at.y - picker.origin.y).clamp(0, picker.size.1 - 1),
        };
        let old = picker.lens;
        picker.lens = picker.place_lens();
        InvalidateRect(hwnd, &old, 0);
        InvalidateRect(hwnd, &picker.lens, 0);
    }
}

unsafe fn paint(hwnd: HWND, picker: &Picker) {
    unsafe {
        let mut ps: PAINTSTRUCT = std::mem::zeroed();
        let dc = BeginPaint(hwnd, &mut ps);
        let r = ps.rcPaint;
        let (w, h) = (r.right - r.left, r.bottom - r.top);
        BitBlt(
            picker.back,
            r.left,
            r.top,
            w,
            h,
            picker.shot,
            r.left,
            r.top,
            SRCCOPY,
        );
        draw_lens(picker);
        BitBlt(dc, r.left, r.top, w, h, picker.back, r.left, r.top, SRCCOPY);
        EndPaint(hwnd, &ps);
    }
}

unsafe fn draw_lens(picker: &Picker) {
    unsafe {
        let dc = picker.back;
        let lens = picker.lens;
        let side = lens.right - lens.left;
        let cell = side / CELLS;
        let color = picker.color();

        // Ekran dışına taşan kısımlar koyu kalsın.
        let panel = CreateSolidBrush(rgb(0x2a, 0x2b, 0x31));
        FillRect(dc, &lens, panel);
        SetStretchBltMode(dc, COLORONCOLOR);
        let half = CELLS / 2;
        StretchBlt(
            dc,
            lens.left,
            lens.top,
            side,
            side,
            picker.shot,
            picker.cursor.x - half,
            picker.cursor.y - half,
            CELLS,
            CELLS,
            SRCCOPY,
        );

        // Ortadaki piksel: her zeminde görünsün diye siyah + beyaz çerçeve.
        let white = CreateSolidBrush(rgb(255, 255, 255));
        let black = CreateSolidBrush(0);
        let center = RECT {
            left: lens.left + half * cell,
            top: lens.top + half * cell,
            right: lens.left + (half + 1) * cell,
            bottom: lens.top + (half + 1) * cell,
        };
        let outer = RECT {
            left: center.left - 1,
            top: center.top - 1,
            right: center.right + 1,
            bottom: center.bottom + 1,
        };
        FrameRect(dc, &outer, black);
        FrameRect(dc, &center, white);

        // Alt şerit: renk kutusu + hex.
        let strip = RECT {
            top: lens.top + side,
            ..lens
        };
        FillRect(dc, &strip, panel);
        let height = strip.bottom - strip.top;
        let pad = height / 4;
        let chip = RECT {
            left: strip.left + pad,
            top: strip.top + pad,
            right: strip.left + height - pad,
            bottom: strip.bottom - pad,
        };
        let fill = CreateSolidBrush(color);
        FillRect(dc, &chip, fill);
        FrameRect(dc, &chip, white);

        let face = wide("Segoe UI Variable Text");
        let font = CreateFontW(
            -(height * 13 / LABEL),
            0,
            0,
            0,
            600,
            0,
            0,
            0,
            1, // DEFAULT_CHARSET
            0,
            0,
            5, // CLEARTYPE_QUALITY
            0,
            face.as_ptr(),
        );
        let old = SelectObject(dc, font);
        SetBkMode(dc, TRANSPARENT as _);
        SetTextColor(dc, rgb(0xf3, 0xf3, 0xf4));
        let mut text: Vec<u16> = hex(color).to_uppercase().encode_utf16().collect();
        let mut area = RECT {
            left: chip.right,
            ..strip
        };
        DrawTextW(
            dc,
            text.as_mut_ptr(),
            text.len() as i32,
            &mut area,
            DT_CENTER | DT_VCENTER | DT_SINGLELINE,
        );
        SelectObject(dc, old);

        FrameRect(dc, &lens, white);
        for object in [panel, white, black, fill, font] {
            DeleteObject(object);
        }
    }
}

unsafe extern "system" fn window_proc(hwnd: HWND, msg: u32, w: WPARAM, l: LPARAM) -> LRESULT {
    unsafe {
        let Some(picker) = (GetWindowLongPtrW(hwnd, GWLP_USERDATA) as *mut Picker).as_mut() else {
            return DefWindowProcW(hwnd, msg, w, l);
        };
        match msg {
            WM_ERASEBKGND => return 1,
            WM_PAINT => paint(hwnd, picker),
            WM_MOUSEMOVE => track(hwnd, picker),
            WM_LBUTTONDOWN => {
                track(hwnd, picker);
                let color = picker.color();
                // Bırakma olayı alttaki pencereye gitmesin diye bırakınca kapanır.
                picker.result = Some(color);
            }
            WM_LBUTTONUP => {
                if let Some(color) = picker.result {
                    picker.finish(Some(color));
                }
            }
            WM_RBUTTONUP => {
                picker.result = None;
                picker.finish(None);
            }
            WM_KEYDOWN => {
                let step = match w as u16 {
                    VK_ESCAPE => {
                        picker.finish(None);
                        None
                    }
                    VK_RETURN => {
                        let color = picker.color();
                        picker.finish(Some(color));
                        None
                    }
                    VK_LEFT => Some((-1, 0)),
                    VK_RIGHT => Some((1, 0)),
                    VK_UP => Some((0, -1)),
                    VK_DOWN => Some((0, 1)),
                    _ => None,
                };
                if let Some((dx, dy)) = step {
                    SetCursorPos(
                        picker.cursor.x + picker.origin.x + dx,
                        picker.cursor.y + picker.origin.y + dy,
                    );
                    track(hwnd, picker);
                }
            }
            // Başka pencereye geçildi ya da ekran düzeni değişti: görüntü artık geçersiz.
            WM_ACTIVATE if (w & 0xFFFF) as u32 == WA_INACTIVE => picker.finish(None),
            WM_DISPLAYCHANGE => picker.finish(None),
            WM_CLOSE => picker.finish(None),
            _ => return DefWindowProcW(hwnd, msg, w, l),
        }
        0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const MONITOR: RECT = RECT {
        left: 0,
        top: 0,
        right: 1920,
        bottom: 1080,
    };

    #[test]
    fn colorref_becomes_css_hex() {
        assert_eq!(hex(rgb(0x12, 0xA3, 0xF4)), "#12a3f4");
        assert_eq!(hex(0), "#000000");
    }

    #[test]
    fn lens_stays_on_the_monitor() {
        for dpi in [96, 144, 192] {
            for (x, y) in [(0, 0), (1919, 0), (0, 1079), (1919, 1079), (960, 540)] {
                let r = lens_rect(POINT { x, y }, MONITOR, dpi);
                assert!(r.left >= 0 && r.top >= 0, "{x},{y}@{dpi}");
                assert!(r.right <= 1920 && r.bottom <= 1080, "{x},{y}@{dpi}");
                // Seçilen pikselin üstünü örtmez.
                assert!(!(r.left <= x && x < r.right && r.top <= y && y < r.bottom));
            }
        }
    }
}
