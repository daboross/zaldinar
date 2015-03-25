//! depends: rand = "0.2.*"
extern crate zaldinar_core;
extern crate rand;
use rand::Rng;
use zaldinar_core::client::PluginRegister;
use zaldinar_core::events::CommandEvent;

const MESSAGES: &'static str = "\
<yes>As I see it, yes
<yes>It is certain
<yes>It is decidedly so
<yes>Most likely
<yes>Outlook good
<yes>Signs point to yes
<yes>One would be wise to think so
<yes>Naturally
<yes>Without a doubt
<yes>Yes
<yes>Yes, definitely
<yes>You may rely on it
Reply hazy, try again
Ask again later
Better not tell you now
Cannot predict now
Concentrate and ask again
You know the answer better than I
Maybe...
<no>You're kidding, right?
<no>Don't count on it
<no>In your dreams
<no>My reply is no
<no>My sources say no
<no>Outlook not so good
<no>Very doubtful";

fn eightball(event: &CommandEvent) {
    if event.args.is_empty() {
        event.client.send_message(event.channel(), "I can't answer if you don't ask.");
        return;
    }
    let messages = MESSAGES.split('\n').collect::<Vec<&str>>();
    let mut rng = rand::thread_rng();
    let message = rng.choose(&messages).unwrap()
                    .replace("<yes>", "\x0305").replace("<no>", "\x0303");
    event.client.send_message(event.channel(), format!("\x01ACTION shakes the magic 8 ball... \x02{}\x01", message));
}

pub fn register(register: &mut PluginRegister) {
    register.register_command("8ball", eightball);
}
