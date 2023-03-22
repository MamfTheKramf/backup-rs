//! Contains functions for scheduling on windows

#[cfg(test)]
mod test {
    use windows::Win32::System::{
        Com::{CoCreateInstance, CLSCTX_INPROC_SERVER, CoInitializeEx, COINIT_MULTITHREADED},
        TaskScheduler::{CLSID_CTaskScheduler, ITaskScheduler},
    };

    #[test]
    fn getting_feet_wet() {
        unsafe {
            match CoInitializeEx(None, COINIT_MULTITHREADED) {
                Ok(_) => (),
                Err(err) => { dbg!(err); },
            }

            let scheduler = CoCreateInstance::<_, ITaskScheduler>(
                &CLSID_CTaskScheduler,
                None,
                CLSCTX_INPROC_SERVER,
            );
            match scheduler {
                Ok(sched) => { dbg!(sched); }
                Err(err) => { dbg!(err); }
            }
        }
    }
}
