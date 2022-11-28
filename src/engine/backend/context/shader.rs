use crate::engine::utils::cstring::cstr;
use ash::vk;
use std::{ffi::CStr, path::Path};
use tracing_unwrap::{OptionExt, ResultExt};
use track::Context;
use walkdir::WalkDir;

const SHADER_ENTRY_NAME: &CStr = unsafe { cstr("main\0") };

pub struct ShaderHandle;

impl ShaderHandle {
    pub fn create_shader_modules(
        device: &ash::Device,
    ) -> Vec<(vk::ShaderModule, vk::ShaderStageFlags)> {
        WalkDir::new(r"src\engine\backend\shaders")
            .into_iter()
            .filter_map(|entry| {
                entry
                    .ok()
                    .and_then(|entry| entry.path().is_file().then(|| entry.path().to_owned()))
            })
            .map(|path| {
                let filename_parts: Vec<&str> = path
                    .file_name()
                    .unwrap_or_log()
                    .to_str()
                    .unwrap()
                    .split('.')
                    .collect();

                let shader_stage_flags = match filename_parts.as_slice() {
                    [_, "vert", _] => vk::ShaderStageFlags::VERTEX,
                    [_, "frag", _] => vk::ShaderStageFlags::FRAGMENT,
                    _ => panic!(
                        "Unknown shader type: {}",
                        path.file_name()
                            .unwrap_or_log()
                            .to_string_lossy()
                            .into_owned()
                    ),
                };

                (
                    Self::create_shader_module(device, &path).unwrap_or_log(),
                    shader_stage_flags,
                )
            })
            .collect()
    }

    fn create_shader_module(device: &ash::Device, path: &Path) -> track::Result<vk::ShaderModule> {
        let mut file = std::fs::File::open(path).track()?;
        let decoded = ash::util::read_spv(&mut file).track()?;

        let shader_module_info = vk::ShaderModuleCreateInfo::default().code(&decoded);
        let shader_module = unsafe {
            device
                .create_shader_module(&shader_module_info, None)
                .track()?
        };

        Ok(shader_module)
    }
}
