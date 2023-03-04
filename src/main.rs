use std::io::Write;
use std::time::{SystemTime, UNIX_EPOCH};

use ash::{vk, Entry};

unsafe extern "system" fn vulkan_debug_utils_callback(
    message_severity: vk::DebugUtilsMessageSeverityFlagsEXT,
    message_type: vk::DebugUtilsMessageTypeFlagsEXT,
    p_callback_data: *const vk::DebugUtilsMessengerCallbackDataEXT,
    _p_user_data: *mut std::ffi::c_void,
) -> vk::Bool32 {
    let message = std::ffi::CStr::from_ptr((*p_callback_data).p_message);
    let message_severity = format!("{:?}", message_severity).to_lowercase();
    let message_type = format!("{:?}", message_type).to_lowercase();

    let mut debug_file = std::fs::OpenOptions::new()
        .append(true)
        .open("debug.txt")
        .unwrap();
    let timestamp = format!(
        "\n\nTIME SINCE UNIX EPOCH (Seconds u64): {}\n", 
        SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs()
    );
    debug_file.write(timestamp.as_bytes()).unwrap();
    let debug_string = format!("[Debug][{}][{}] {:?}", message_severity, message_type, message);
    debug_file.write(debug_string.as_bytes()).unwrap();
    vk::FALSE
}

fn main() {
    let engine_name = std::ffi::CString::new("Vulkan Tutorial").unwrap();
    let app_name = std::ffi::CString::new("The Black Window").unwrap();
    let engine_version = vk::make_api_version(0, 1, 0, 0);
    let application_version = vk::make_api_version(0, 1, 0, 0);
    let api_version = vk::make_api_version(1, 3, 0, 0);

    let entry = unsafe { Entry::load().unwrap() };

    let app_info = vk::ApplicationInfo::builder()
        .application_name(&app_name)
        .engine_name(&engine_name)
        .engine_version(engine_version)
        .application_version(application_version)
        .api_version(api_version);

    let layer_names: Vec<std::ffi::CString> =
        vec![std::ffi::CString::new("VK_LAYER_KHRONOS_validation").unwrap()];
    let layer_name_pointers: Vec<*const i8> = layer_names
        .iter()
        .map(|layer_name| layer_name.as_ptr())
        .collect();
    let extension_name_pointers: Vec<*const i8> =
        vec![ash::extensions::ext::DebugUtils::name().as_ptr()];

    let mut debug_create_info = vk::DebugUtilsMessengerCreateInfoEXT {
        message_severity: vk::DebugUtilsMessageSeverityFlagsEXT::WARNING
            | vk::DebugUtilsMessageSeverityFlagsEXT::VERBOSE
            | vk::DebugUtilsMessageSeverityFlagsEXT::INFO
            | vk::DebugUtilsMessageSeverityFlagsEXT::ERROR,
        message_type: vk::DebugUtilsMessageTypeFlagsEXT::GENERAL
            | vk::DebugUtilsMessageTypeFlagsEXT::PERFORMANCE
            | vk::DebugUtilsMessageTypeFlagsEXT::VALIDATION,
        pfn_user_callback: Some(vulkan_debug_utils_callback),
        ..Default::default()
    };
    
    let instance_create_info = vk::InstanceCreateInfo::builder()
        .push_next(&mut debug_create_info)
        .application_info(&app_info)
        .enabled_layer_names(&layer_name_pointers)
        .enabled_extension_names(&extension_name_pointers);

    let instance = unsafe { entry.create_instance(&instance_create_info, None).unwrap() };

    let physical_devices = unsafe { instance.enumerate_physical_devices().unwrap() };

    let mut graphic_operation_supporting_devices = Vec::<usize>::new();
    let mut compute_operation_supporting_devices = Vec::<usize>::new();
    let mut transfer_operation_supporting_devices = Vec::<usize>::new();
    let mut sparse_binding_operation_supporting_devices = Vec::<usize>::new();

    for device in physical_devices {
        let queue_properity_vec = unsafe { instance.get_physical_device_queue_family_properties(device) };
        for (index, queue_properties) in queue_properity_vec.iter().enumerate() {
            if queue_properties.queue_count > 0 && queue_properties.queue_flags.contains(vk::QueueFlags::GRAPHICS) {
                graphic_operation_supporting_devices.push(index);
            }
            if queue_properties.queue_count > 0 && queue_properties.queue_flags.contains(vk::QueueFlags::COMPUTE) {
                compute_operation_supporting_devices.push(index);
            }
            if queue_properties.queue_count > 0 && queue_properties.queue_flags.contains(vk::QueueFlags::TRANSFER) {
                transfer_operation_supporting_devices.push(index);
            }
            if queue_properties.queue_count > 0 && queue_properties.queue_flags.contains(vk::QueueFlags::SPARSE_BINDING) {
                sparse_binding_operation_supporting_devices.push(index);
            }
        }
    }

    println!("GRAPHICS: {:?}", graphic_operation_supporting_devices);
    println!("COMPUTE: {:?}", compute_operation_supporting_devices);
    println!("TRANSFER: {:?}", transfer_operation_supporting_devices);
    println!("SPARSE_BINDING: {:?}", sparse_binding_operation_supporting_devices);

    let debug_utils = ash::extensions::ext::DebugUtils::new(&entry, &instance);

    let utils_messenger =
        unsafe { debug_utils.create_debug_utils_messenger(&debug_create_info, None).unwrap() };

    unsafe { 
        debug_utils.destroy_debug_utils_messenger(utils_messenger, None);
        instance.destroy_instance(None);
    };
}