//! Contains functions for scheduling on windows

#[cfg(test)]
mod test {
    use windows::{
        core::{ComInterface, BSTR},
        Win32::System::{
            Com::{
                CoCreateInstance, CoInitializeEx, CoUninitialize, CLSCTX_INPROC_SERVER,
                COINIT_MULTITHREADED, VARIANT, CLSIDFromProgID,
            },
            TaskScheduler::{
                IExecAction, ITaskService, ITimeTrigger, TASK_ACTION_EXEC,
                TASK_CREATE_OR_UPDATE, TASK_LOGON_INTERACTIVE_TOKEN, TASK_TRIGGER_TIME,
            },
        }, w,
    };

    fn err_transform<T, E: std::error::Error>(err: E) -> Result<T, ()> {
        dbg!(err);
        unsafe {
            CoUninitialize();
        }
        Err(())
    }

    #[test]
    fn getting_feet_wet() -> Result<(), ()> {
        let folder_name = "\\\0";
        let task_name = "test_task\0";

        unsafe {
            #[allow(non_snake_case)]
            let CLSID_ITaskService = CLSIDFromProgID(w!("Schedule.Service"))
            .or_else(err_transform)?;

            println!("CoInitializeEx...");
            match CoInitializeEx(None, COINIT_MULTITHREADED) {
                Ok(_) => (),
                Err(err) => {
                    dbg!(err);
                }
            }

            println!("CoCreateInstance...");
            let service: ITaskService = CoCreateInstance(
                &CLSID_ITaskService,
                None,
                CLSCTX_INPROC_SERVER,
            )
            .or_else(err_transform)?;

            println!("ITaskService.Connect...");
            service
                .Connect(
                    VARIANT::default(),
                    VARIANT::default(),
                    VARIANT::default(),
                    VARIANT::default(),
                )
                .or_else(err_transform)?;

            println!("ITaskService.GetFolder...");
            let task_folder = service
                .GetFolder(&BSTR::from(folder_name))
                .or_else(err_transform)?;

            // println!("ITaskFolder.DeleteTask...");
            // task_folder
            //     .DeleteTask(&BSTR::from(task_name), 0)
            //     .or_else(err_transform)?;

            println!("ITaskService.NewTask...");
            let task = service.NewTask(0).or_else(err_transform)?;

            println!("ITaskDefinition.RegistrationInfo...");
            let reg_info = task.RegistrationInfo().or_else(err_transform)?;

            println!("IRegistrationInfo.SetAuthor...");
            reg_info
                .SetAuthor(&BSTR::from("MamfTheKramf\n"))
                .or_else(err_transform)?;
            println!("IRegistrationInfo.SetDescription...");
            reg_info
                .SetDescription(&BSTR::from("MamfTheKramf\n"))
                .or_else(err_transform)?;
            
            println!("ITaskDefinition.Triggers...");
            let trigger_collection = task.Triggers().or_else(err_transform)?;
            

            println!("ITriggerCollection.Create...");
            let time_trigger = trigger_collection
                .Create(TASK_TRIGGER_TIME)
                .or_else(err_transform)?;
            
            println!("ITrigger.Cast...");
            let time_trigger = time_trigger
                .cast::<ITimeTrigger>()
                .or_else(err_transform)?;
            
            println!("SetId...");
            time_trigger
                .SetId(&BSTR::from("Trigger1\0"))
                .or_else(err_transform)?;
            println!("SetEndBoundary...");
            time_trigger
                .SetEndBoundary(&BSTR::from("2030-12-31T23:59:59\0"))
                .or_else(err_transform)?;
            println!("SetStartBoundary...");
            time_trigger
                .SetStartBoundary(&BSTR::from("2023-03-22T20:27:00\0"))
                .or_else(err_transform)?;
            
            println!("task.Actions...");
            let action_collection = task.Actions().or_else(err_transform)?;
            println!("action_collection.Create...");
            let action = action_collection
                .Create(TASK_ACTION_EXEC)
                .or_else(err_transform)?;
            println!("action.Cast...");
            let action = action
                .cast::<IExecAction>()
                .or_else(err_transform)?;
            println!("action.SetPath...");
            action.SetPath(&BSTR::from("F:\\Projekte\\Rust\\backup\\backupper\\test_dir\\tasks\\openStackOverflow.bat\0"))
                .or_else(err_transform)?;
            
            println!("RegisterTaskDefinition...");
            task_folder
                .RegisterTaskDefinition(
                    &BSTR::from(task_name),
                    &task,
                    TASK_CREATE_OR_UPDATE.0,
                    VARIANT::default(),
                    VARIANT::default(),
                    TASK_LOGON_INTERACTIVE_TOKEN,
                    VARIANT::default(),
                )
                .or_else(err_transform)?;

            println!("CoUninitialize");
            // CoUninitialize()
        }
        println!("Task should be initialized now");
        Ok(())
    }
}
