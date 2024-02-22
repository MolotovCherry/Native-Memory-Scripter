use std::sync::atomic::{AtomicBool, Ordering};

use windows::{
    core::PCWSTR,
    Win32::System::Console::{
        AllocConsole, FreeConsole, GetStdHandle, SetConsoleMode, SetConsoleTitleW,
        ENABLE_ECHO_INPUT, ENABLE_INSERT_MODE, ENABLE_LINE_INPUT, ENABLE_PROCESSED_INPUT,
        ENABLE_PROCESSED_OUTPUT, ENABLE_QUICK_EDIT_MODE, ENABLE_VIRTUAL_TERMINAL_INPUT,
        ENABLE_VIRTUAL_TERMINAL_PROCESSING, ENABLE_WRAP_AT_EOL_OUTPUT, STD_INPUT_HANDLE,
        STD_OUTPUT_HANDLE,
    },
};

static ALLOCATED: AtomicBool = AtomicBool::new(false);

pub fn alloc_console() -> ::windows::core::Result<()> {
    let allocated = ALLOCATED.load(Ordering::Acquire);
    if allocated {
        return Ok(());
    }

    unsafe {
        AllocConsole()?;
    }

    let handle = unsafe { GetStdHandle(STD_OUTPUT_HANDLE)? };

    unsafe {
        SetConsoleMode(
            handle,
            ENABLE_PROCESSED_OUTPUT
                | ENABLE_WRAP_AT_EOL_OUTPUT
                | ENABLE_VIRTUAL_TERMINAL_PROCESSING,
        )?;
    }

    let handle = unsafe { GetStdHandle(STD_INPUT_HANDLE)? };

    unsafe {
        SetConsoleMode(
            handle,
            ENABLE_ECHO_INPUT
                | ENABLE_INSERT_MODE
                | ENABLE_LINE_INPUT
                | ENABLE_PROCESSED_INPUT
                | ENABLE_VIRTUAL_TERMINAL_INPUT
                | ENABLE_QUICK_EDIT_MODE,
        )?;
    }

    let title = "Native Memory Scripter Debug Console"
        .encode_utf16()
        .chain(std::iter::once(0u16))
        .collect::<Vec<_>>();

    unsafe {
        SetConsoleTitleW(PCWSTR(title.as_ptr()))?;
    }

    ALLOCATED.store(true, Ordering::Release);

    print_intro();

    Ok(())
}

pub fn free_console() -> ::windows::core::Result<()> {
    let allocated = ALLOCATED.load(Ordering::Acquire);
    if !allocated {
        return Ok(());
    }

    unsafe {
        FreeConsole()?;
    }

    ALLOCATED.store(false, Ordering::Release);

    Ok(())
}

fn print_intro() {
    let version = env!("CARGO_PKG_VERSION");
    // short sha
    let sha = &env!("VERGEN_GIT_SHA")[..8];
    let built = env!("VERGEN_BUILD_DATE");
    let debug = cfg!(debug_assertions);

    println!(
        r#"
********************************************************************************
*                                                                              *
*                     Native Memory Scripter Debug Console                     *
*                                                                              *
********************************************************************************

Version {version}@{sha} (debug: {debug}) built on {built}
"#
    );
}
