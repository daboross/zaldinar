extern crate "zaldinar-core" as zaldinar;

{{extern_crate_lines}}

pub fn register(register: &mut zaldinar::client::PluginRegister) {
    {{register_lines}}
}

// TODO: Implement commands from http://sprunge.us/KSSH
