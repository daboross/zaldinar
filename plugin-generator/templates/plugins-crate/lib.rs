extern crate zaldinar_core;

{{extern_crate_lines}}

pub fn register(register: &mut zaldinar_core::client::PluginRegister) {
    {{register_lines}}
}

// TODO: Implement commands from http://sprunge.us/KSSH
