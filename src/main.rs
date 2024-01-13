#![windows_subsystem = "windows"] // Use this trait attribute to not allocate console window
use std::ffi::c_void;

use windows::core::*;
use windows::Win32::{
    Foundation::*,
    UI::WindowsAndMessaging::*,
    Graphics::Gdi::UpdateWindow,
    System::LibraryLoader::GetModuleHandleExW,
};

const ID_OUTPUT: isize = 1;
const ID_INCREASE: isize = 2;
const ID_DECREASE: isize = 3;

struct Counter {
    counter: i32,
    output_handle: HWND,
}

fn main() -> Result<()> {
    let lparam: *mut Counter = Box::leak(Box::new(Counter{
        counter: 1,
        output_handle: HWND(0),
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
            w!("Dead simple counter"),
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

fn add_delta_and_print(parent: HWND, delta: i32) {
    unsafe {
        let counter_ptr = GetWindowLongPtrW(parent, GWLP_USERDATA) as *mut Counter;
        if let Some(c) = counter_ptr.as_mut() {
            c.counter += delta;
            // println!("counter: {}", c.counter);
            let res = SetWindowTextW(c.output_handle, string_to_pcwstr(c.counter.to_string()));
            debug_assert!(res.is_ok());
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
            create_increase_btn(hwnd);
            create_decrease_btn(hwnd);

            let cs = lparam.0 as *const CREATESTRUCTW;
            let counter = (*cs).lpCreateParams as *mut Counter;
            (*counter).output_handle = create_output_edit(hwnd, (*counter).counter);

            SetWindowLongPtrW(hwnd, GWLP_USERDATA, counter as _);

            LRESULT(0)
        },
        WM_DESTROY => {
            PostQuitMessage(0);
            LRESULT(0)
        },
        WM_COMMAND => {
            let command = HMENU(wparam.0 as isize);
            match command {
                HMENU(ID_INCREASE) => {
                    add_delta_and_print(hwnd, 1);
                },
                HMENU(ID_DECREASE) => {
                    add_delta_and_print(hwnd, -1);
                },
                _ => {}
            }
            return LRESULT(0);
        },
        _ => DefWindowProcW(hwnd, msg, wparam, lparam)
    }
}


fn create_decrease_btn(hwnd: HWND) -> HWND {
    unsafe {
        let res = CreateWindowExW(
            WS_EX_LEFT,
            w!("BUTTON"),
            w!("Decrease"),
            WS_VISIBLE | WS_CHILD,
            220,
            0,
            200, 40,
            hwnd,
            HMENU(ID_DECREASE),
            HINSTANCE(GetWindowLongPtrW(hwnd, GWLP_HINSTANCE)),
            None,
        );
        debug_assert_ne!(res, HWND(0));
        res
    }
}

fn create_increase_btn(hwnd: HWND) -> HWND {
    unsafe {
        let res = CreateWindowExW(
            WS_EX_LEFT,
            w!("BUTTON"),
            w!("Increase"),
            WS_VISIBLE | WS_CHILD,
            0,
            0,
            200, 40,
            hwnd,
            HMENU(ID_INCREASE),
            HINSTANCE(GetWindowLongPtrW(hwnd, GWLP_HINSTANCE)),
            None,
        );
        debug_assert_ne!(res, HWND(0));
        res
    }
}

fn create_output_edit(hwnd: HWND, count: i32) -> HWND {
    unsafe {
        let res = CreateWindowExW(
            WS_EX_LEFT,
            w!("EDIT"),
            string_to_pcwstr(count.to_string()),
            WS_CHILD | WS_VISIBLE | WS_BORDER | WS_DISABLED,
            0,
            100,
            500, 40,
            hwnd,
            HMENU(ID_OUTPUT),
            HINSTANCE(GetWindowLongPtrW(hwnd, GWLP_HINSTANCE)),
            None,
        );
        debug_assert_ne!(res, HWND(0));
        res
    }
}


