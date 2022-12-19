use crate::engine::utils::cstring::cstr;
use ash::vk;
use smallvec::SmallVec;
use std::{ffi::CStr, path::Path};
use tracing_unwrap::OptionExt;
use track::Context;
use walkdir::WalkDir;

pub const SHADER_ENTRY_NAME: &CStr = cstr!("main");

pub struct ShaderHandle {
    pub shader_modules: SmallVec<[(vk::ShaderModule, vk::ShaderStageFlags); 2]>,
}

impl ShaderHandle {
    pub fn new(device: &ash::Device) -> Self {
        let shader_modules = WalkDir::new(r"src\engine\renderer\shaders\spv")
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

                print!("TEST");

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
                    Self::create_shader_module(device, &path).unwrap(),
                    shader_stage_flags,
                )
            })
            .collect::<SmallVec<[(vk::ShaderModule, vk::ShaderStageFlags); 2]>>();

        assert!(
            !shader_modules.is_empty(),
            "Found no shaders in the specified path"
        );

        Self { shader_modules }
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
