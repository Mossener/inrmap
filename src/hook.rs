//! Windows Hook 模块 - 低级键盘钩子实现

use crate::mapping::{lookup, Mapping};
use std::ptr;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use windows::Win32::Foundation::*;
use windows::Win32::System::LibraryLoader::GetModuleHandleW;
use windows::Win32::UI::Input::KeyboardAndMouse::*;
use windows::Win32::UI::WindowsAndMessaging::*;

/// 全局钩子句柄
static mut HOOK: HHOOK = HHOOK(ptr::null_mut());

/// 全局运行标志
static RUNNING: AtomicBool = AtomicBool::new(true);

/// 模拟按键（按下+释放）
pub fn send_key(vk: u32) {
    unsafe {
        let down = INPUT {
            r#type: INPUT_KEYBOARD,
            Anonymous: INPUT_0 {
                ki: KEYBDINPUT {
                    wVk: VIRTUAL_KEY(vk as u16),
                    wScan: 0,
                    dwFlags: KEYBD_EVENT_FLAGS(0),
                    time: 0,
                    dwExtraInfo: 0,
                },
            },
        };

        let up = INPUT {
            r#type: INPUT_KEYBOARD,
            Anonymous: INPUT_0 {
                ki: KEYBDINPUT {
                    wVk: VIRTUAL_KEY(vk as u16),
                    wScan: 0,
                    dwFlags: KEYEVENTF_KEYUP,
                    time: 0,
                    dwExtraInfo: 0,
                },
            },
        };

        let _ = SendInput(&[down, up], std::mem::size_of::<INPUT>() as i32);
    }
}

/// 初始化钩子
pub fn init() -> Result<(), String> {
    unsafe {
        let h_instance = GetModuleHandleW(None).map_err(|e| e.to_string())?;
        
        HOOK = SetWindowsHookExW(
            WH_KEYBOARD_LL,
            Some(hook_proc),
            h_instance,
            0,
        ).map_err(|e| format!("SetWindowsHookExW failed: {}", e))?;

        if HOOK.0.is_null() {
            return Err("Hook install failed (null)".to_string());
        }
    }
    
    println!("[初始化] 键盘钩子已安装");
    Ok(())
}

/// 清理钩子
pub fn cleanup() {
    unsafe {
        if !HOOK.0.is_null() {
            let _ = UnhookWindowsHookEx(HOOK);
            println!("[清理] 键盘钩子已卸载");
        }
    }
}

/// 停止运行
pub fn stop() {
    RUNNING.store(false, Ordering::SeqCst);
}

/// 钩子回调函数
unsafe extern "system" fn hook_proc(
    code: i32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    if code >= 0 && ENABLED.load(Ordering::SeqCst) {
        let kb = *(lparam.0 as *const KBDLLHOOKSTRUCT);
        let vk = kb.vkCode;
        let flags = kb.flags;
        
        // 检查是否是注入事件（我们自己模拟的），如果是则放行
        // LLKHF_INJECTED = 0x01
        if (flags.0 & 0x01) != 0 {
            return CallNextHookEx(HOOK, code, wparam, lparam);
        }

        // 获取映射表
        let map_ptr = MAP.load(Ordering::SeqCst);
        if map_ptr != 0 {
            let map = &*(map_ptr as *const Mapping);
            
            match wparam.0 as u32 {
                WM_KEYDOWN | WM_SYSKEYDOWN => {
                    if let Some(new_vk) = lookup(map, vk) {
                        send_key(new_vk);
                        return LRESULT(1); // 阻止原键
                    }
                }
                WM_KEYUP | WM_SYSKEYUP => {
                    if lookup(map, vk).is_some() {
                        return LRESULT(1); // 阻止原键
                    }
                }
                _ => {}
            }
        }
    }

    CallNextHookEx(HOOK, code, wparam, lparam)
}

/// 全局映射表指针
static MAP: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);

/// 全局启用标志
static ENABLED: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(true);

/// 设置启用状态
pub fn set_enabled(enabled: bool) {
    ENABLED.store(enabled, Ordering::SeqCst);
}

/// 检查是否启用
pub fn is_enabled() -> bool {
    ENABLED.load(Ordering::SeqCst)
}

/// 设置映射表
pub fn set_mapping(map: Mapping) {
    let boxed = Box::new(map);
    let ptr = Box::into_raw(boxed) as usize;
    MAP.store(ptr, Ordering::SeqCst);
}

/// 消息循环
pub fn run_message_loop() {
    unsafe {
        let mut msg = MSG::default();

        while RUNNING.load(Ordering::SeqCst)
            && GetMessageW(&mut msg, HWND(ptr::null_mut()), 0, 0).as_bool()
        {
            TranslateMessage(&msg);
            DispatchMessageW(&msg);
        }
    }
}