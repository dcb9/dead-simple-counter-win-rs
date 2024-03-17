#![windows_subsystem = "windows"] // Use this trait attribute to not allocate console window
use std::ffi::c_void;

use windows::core::*;
use windows::Win32::Graphics::Gdi::{
    BeginPaint, CreatePen, CreateSolidBrush, DrawTextExW, EndPaint, LineTo, MoveToEx, Rectangle, RedrawWindow, SelectObject, DT_SINGLELINE, PS_NULL, PS_SOLID, RDW_ERASE, RDW_INVALIDATE};
use windows::Win32::{
    Foundation::*,
    UI::WindowsAndMessaging::*,
    Graphics::Gdi::UpdateWindow,
    System::LibraryLoader::GetModuleHandleExW,
};

const ID_INPUT: isize = 1;

struct Thermometer {
    temperature: f32,
    input_box: HWND,
}

fn main() -> Result<()> {
    let lparam: *mut Thermometer = Box::leak(Box::new(Thermometer{
        temperature: 18.0,
        input_box: HWND(0),
    }));

    unsafe {
        let mut instance = Default::default();
        GetModuleHandleExW(0, None, &mut instance)?;

        let wnd_class = WNDCLASSEXW {
            hCursor: LoadCursorW(None, IDC_ARROW)?,
            hInstance: instance.into(),
            lpszClassName: w!("my_window"),
            lpfnWndProc: Some(window_proc),
            cbSize: std::mem::size_of::<WNDCLASSEXW>() as u32,

            ..Default::default()
        };

        let atom = RegisterClassExW(&wnd_class);
        debug_assert!(atom != 0);

        let hwnd = CreateWindowExW(
            Default::default(),
            w!("my_window"),
            w!("Thermometer"),
            WS_OVERLAPPEDWINDOW,
            CW_USEDEFAULT,
            CW_USEDEFAULT,
            CW_USEDEFAULT,
            CW_USEDEFAULT,
            None,
            None,
            instance,
            Some(lparam as *mut c_void),
        );

        ShowWindow(hwnd, SW_SHOW);
        UpdateWindow(hwnd);

        let mut msg = MSG::default();
        while GetMessageW(&mut msg, None, 0, 0).into() {
            TranslateMessage(&msg);
            DispatchMessageW(&msg);
        }

        Ok(())
    }
}

fn on_input_change(parent: HWND) {
    unsafe {
        let thermometer_ptr = GetWindowLongPtrW(parent, GWLP_USERDATA) as *mut Thermometer;
        if let Some(c) = thermometer_ptr.as_mut() {
            // Allocate a buffer for the text. Add 1 for the null terminator.
            let mut bytes: [u16; 500] = [0; 500];

            let len = GetWindowTextW(c.input_box, &mut bytes);
            let temperature = String::from_utf16_lossy(&bytes[..len as usize]);
            println!("{}", temperature);
            c.temperature = match temperature.parse() {
                Ok(v) => v,
                Err(_) => 0.0 // or whatever error handling
            };
            println!("temperature: {}", c.temperature);

            RedrawWindow(parent, None, None, RDW_INVALIDATE | RDW_ERASE);
        }
    }
}

fn string_to_pcwstr(s: String) -> PCWSTR {
    let lpstr = s.encode_utf16().chain(::std::iter::once(0)).collect::<Vec<u16>>().as_mut_ptr();
    PCWSTR::from_raw(lpstr)
}

unsafe extern "system" fn window_proc(hwnd: HWND, msg: u32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
    match msg {
        WM_CREATE => {
            let cs = lparam.0 as *const CREATESTRUCTW;
            let thermometer = (*cs).lpCreateParams as *mut Thermometer;
            (*thermometer).input_box = create_input_edit(hwnd, (*thermometer).temperature);

            SetWindowLongPtrW(hwnd, GWLP_USERDATA, thermometer as _);

            LRESULT(0)
        },
        WM_PAINT => {
            let thermometer = (GetWindowLongPtrW(hwnd, GWLP_USERDATA) as *mut Thermometer).as_mut().unwrap();

            let mut ps = Default::default();
            let hdc = BeginPaint(hwnd, &mut ps);

            let draw_text = |text: String, rect: RECT| {
                let mut wide_text: Vec<u16> = text.encode_utf16().collect();
                let s = wide_text.as_mut_slice();

                DrawTextExW(
                    hdc,
                    s,
                    &mut rect.clone(),
                    DT_SINGLELINE,
                    None,
                );
            };
            draw_text("Temperature °C".to_string(), RECT {
                left: 30,
                top: 50,
                right: 300,
                bottom: 100,
            });


            let draw_vessel = |x1:i32, y1:i32, x2:i32, y2:i32| {
                let hbrush1 = CreateSolidBrush(COLORREF(0xffffff));
                let hpen = CreatePen(PS_SOLID, 4, COLORREF(0x000000));
                SelectObject(hdc, hpen);
                SelectObject(hdc, hbrush1);
                Rectangle(hdc, x1, y1, x2, y2);

                if thermometer.temperature >= 18.0 && thermometer.temperature <= 21.0 {
                    let hbrush1 = CreateSolidBrush(COLORREF(0x0000ff));
                    let hpen = CreatePen(PS_NULL, 0, COLORREF(0x000000));
                    SelectObject(hdc, hpen);
                    SelectObject(hdc, hbrush1);
                    Rectangle(hdc, x1+2, y2-30-((thermometer.temperature-18.0)*100.0) as i32, x2-2, y2-2);
                }
            };
            draw_vessel(50, 100, 100, 450);

            let pen1 = CreatePen(PS_SOLID, 1, COLORREF(0x000000));
            let pen2 = CreatePen(PS_SOLID, 1, COLORREF(0x0000ff));
            let pen3 = CreatePen(PS_SOLID, 1, COLORREF(0x000000));
            // 18 ~ 21 degree
            let draw_lines = | | {
                let (scale_len1, scale_len2, scale_len3) = (25, 35, 45);
                let x = 100;

                for i in 0..31 {
                    let y = 120 + i * 10;
                    if i%10 == 0 {
                        SelectObject(hdc, pen3);
                        MoveToEx(hdc, x, y, None);
                        LineTo(hdc, x+scale_len3, y);
                        MoveToEx(hdc, x, y+1, None);
                        LineTo(hdc, x+scale_len3, y+1);
                        draw_text(format!("{:.1}", (21-(i/10)) as f32), RECT {
                            left: x+scale_len3+10,
                            top: y-6,
                            right: x+scale_len3+300,
                            bottom: y+300,
                        });
                        // println!("y {}", y);
                    } else if i%5 == 0 {
                        SelectObject(hdc, pen2);
                        MoveToEx(hdc, x, y, None);
                        LineTo(hdc, x+scale_len2, y);
                    } else {
                        SelectObject(hdc, pen1);
                        MoveToEx(hdc, x, y, None);
                        LineTo(hdc, x+scale_len1, y);
                    }
                }
            };
            draw_lines();

            draw_text("Enter a value to be displayed".to_string(), RECT {
                left: 30,
                top: 480,
                right: 300,
                bottom: 580,
            });

            if thermometer.temperature <18.0 || thermometer.temperature > 21.0 {
                draw_text("Please enter a number from 18.0 to 21.0".to_string(), RECT {
                    left: 250,
                    top: 520,
                    right: 600,
                    bottom: 580,
                });
            } else {
                draw_text(" ".repeat(256).to_string(), RECT {
                    left: 250,
                    top: 520,
                    right: 600,
                    bottom: 580,
                });
            }

            EndPaint(hwnd, &mut ps);

            LRESULT(0)
        },
        WM_DESTROY => {
            PostQuitMessage(0);
            LRESULT(0)
        },
        WM_COMMAND => {
            on_input_change(hwnd);
            return LRESULT(0);
        },
        _ => DefWindowProcW(hwnd, msg, wparam, lparam)
    }
}

fn create_input_edit(hwnd: HWND, temperature: f32) -> HWND {
    unsafe {
        let res = CreateWindowExW(
            WS_EX_LEFT,
            w!("EDIT"),
            string_to_pcwstr(temperature.to_string()),
            WS_CHILD | WS_VISIBLE | WS_BORDER ,
            250,
            475,
            60, 30,
            hwnd,
            HMENU(ID_INPUT),
            HINSTANCE(GetWindowLongPtrW(hwnd, GWLP_HINSTANCE)),
            None,
        );
        debug_assert_ne!(res, HWND(0));
        res
    }
}
