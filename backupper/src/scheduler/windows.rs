//! Contains functions for scheduling on windows

use chrono::NaiveDateTime;
use uuid::Uuid;
use windows::{
    core::{ComInterface, BSTR},
    w,
    Win32::{
        Foundation::VARIANT_BOOL,
        System::{
            Com::{
                CLSIDFromProgID, CoCreateInstance, CoInitializeEx, CLSCTX_INPROC_SERVER,
                COINIT_MULTITHREADED, VARIANT,
            },
            TaskScheduler::{
                IActionCollection, IExecAction, IRegistrationInfo, ITaskFolder, ITaskService,
                ITimeTrigger, ITriggerCollection, TASK_ACTION_EXEC, TASK_CREATE_OR_UPDATE,
                TASK_LOGON_INTERACTIVE_TOKEN, TASK_TRIGGER_TIME,
            },
        },
    },
};

const ROOT_FOLDER: &str = "\\\0";

fn transform_err<T>(msg: &str) -> Result<T, String> {
    Err(msg.to_string())
}

/// Gets an [ITaskFolder] for the folder with the current name.
/// If such a folder doesn't exist, it is attempted to create one.
///
/// # Parameters
/// - `service`: [ITaskService] to call [ITaskService::GetFolder] on. It is assumed, that [ITaskService::Connect] has already been called,
/// - `folder_name`: [BSTR] of the folder name that shall be returned / created
unsafe fn get_task_folder(service: &ITaskService, folder_name: &BSTR) -> Result<ITaskFolder, ()> {
    // Return correct folder if it exists
    if let Ok(folder) = service.GetFolder(folder_name) {
        return Ok(folder);
    }
    // create folder new
    let root_folder = service
        .GetFolder(&BSTR::from(ROOT_FOLDER))
        .or(Err(()))?;
    root_folder
        .CreateFolder(folder_name, windows::Win32::System::Com::VARIANT::default())
        .or(Err(()))
}

unsafe fn set_registration_info(reg_info: &IRegistrationInfo) -> Result<(), &str> {
    reg_info
        .SetAuthor(&BSTR::from("backup-rs\0"))
        .or(Err("Couldn't set author"))?;
    reg_info
        .SetDescription(&BSTR::from(
            "Starts the backup program at the specified time\0",
        ))
        .or(Err("Couldn't set description"))?;
    Ok(())
}

/// Creates a new [ITimeTrigger] and sets its `StartBoundary` to the provided [NaiveDateTime].
///
/// # Errors
/// If something goes wrong, an [Err] describing the issue is returned.
unsafe fn set_up_time_trigger(
    trigger_collection: &ITriggerCollection,
    trigger_time: NaiveDateTime,
) -> Result<(), &str> {
    let mut trigger_count: i32 = 0;
    trigger_collection
        .Count(&mut trigger_count)
        .or(Err("Couldn't get number of triggers"))?;

    let time_trigger = trigger_collection
        .Create(TASK_TRIGGER_TIME)
        .or(Err("Couldn't create new trigger"))?;

    let time_trigger = time_trigger
        .cast::<ITimeTrigger>()
        .or(Err("Couldn't cast trigger"))?;

    time_trigger
        .SetId(&BSTR::from(format!("Trigger{}\0", trigger_count)))
        .or(Err("Couldn't set trigger ID"))?;

    time_trigger
        .SetEndBoundary(&BSTR::from("3000-12-31T23:59:59\0"))
        .or(Err("Couldn't set EndBoundary"))?;

    let date_string = trigger_time.format("%Y-%m-%dT%H:%M:%S").to_string() + "\0";

    time_trigger
        .SetStartBoundary(&BSTR::from(date_string))
        .or(Err("Couldn't set StartBoundary"))?;

    Ok(())
}

/// Creates a new action calling the executable or the provided [Uuid].
///
/// # Errors
/// If something goes wrong, an [Err] describing the issue is returned.
unsafe fn set_up_action(action_collection: &IActionCollection, uuid: Uuid) -> Result<(), &str> {
    let action = action_collection
        .Create(TASK_ACTION_EXEC)
        .or(Err("Couldn't create new action"))?;

    let action = action
        .cast::<IExecAction>()
        .or(Err("Couldn't cast action"))?;

    let err = Err("Couldn't get path to current exectable");
    let path = format!("{:?}\0", std::env::current_exe().or(err)?);
    action
        .SetPath(&BSTR::from(path))
        .or(Err("Couldn't set executable path"))?;

    let args = format!("-u {}", uuid.as_hyphenated().to_string());
    action
        .SetArguments(&BSTR::from(args))
        .or(Err("Couldn't set arguments for action"))?;

    Ok(())
}

/// Schedules a backup for the profile with the given [Uuid] at the provided [NaiveDateTime].
///
/// # Errors
/// Returns an [Err] describing what went wrong if there was an issue.
pub fn schedule_backup(uuid: Uuid, date_time: NaiveDateTime) -> Result<(), String> {
    let folder_name = BSTR::from("\\backup-rs\0");
    let task_name = BSTR::from(uuid.as_hyphenated().to_string() + "\0");

    unsafe {
        #[allow(non_snake_case)]
        let CLSID_ITaskService = CLSIDFromProgID(w!("Schedule.Service"))
            .or_else(|_| transform_err("Couldn't get CLSID of Schedule.Service"))?;

        if let Err(_) = CoInitializeEx(None, COINIT_MULTITHREADED) {
            return transform_err("Couldn't initialize");
        }

        let service: ITaskService =
            CoCreateInstance(&CLSID_ITaskService, None, CLSCTX_INPROC_SERVER)
                .or_else(|_| transform_err("Couldn't get ITaskService"))?;

        service
            .Connect(
                VARIANT::default(),
                VARIANT::default(),
                VARIANT::default(),
                VARIANT::default(),
            )
            .or_else(|_| transform_err("Couldn't connect ITaskService"))?;

        let task_folder = get_task_folder(&service, &BSTR::from(folder_name))
            .or_else(|_| transform_err("Couldn't create task folder"))?;

        let task = service
            .NewTask(0)
            .or_else(|_| transform_err("Couldn't create new Task"))?;

        let reg_info = task
            .RegistrationInfo()
            .or_else(|_| transform_err("Couldn't get RegistrationInfo"))?;

        set_registration_info(&reg_info).or_else(|msg| transform_err(msg))?;

        let trigger_collection = task
            .Triggers()
            .or_else(|_| transform_err("Couldn't get trigger collection"))?;

        set_up_time_trigger(&trigger_collection, date_time).or_else(|msg| transform_err(msg))?;

        let settings = task
            .Settings()
            .or_else(|_| transform_err("Couldn't get task settings"))?;

        settings
            .SetStartWhenAvailable(VARIANT_BOOL(1))
            .or_else(|_| transform_err("Couldn't enable starting when trigger time was missed"))?;

        let action_collection = task
            .Actions()
            .or_else(|_| transform_err("Couldn't get action cllection"))?;

        set_up_action(&action_collection, uuid).or_else(|msg| transform_err(msg))?;

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
            .or_else(|_| transform_err("Couldn't register task definition"))?;
    }
    Ok(())
}
