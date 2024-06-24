use gloo::timers::callback::Timeout;
use log::debug;
use molt_forked::prelude::*;
use std::mem;
use yew::prelude::*;
use yew_icons::{Icon, IconId};

pub struct AppCtx {
    num: usize,
}
pub struct App {
    input_div_ref: NodeRef,
    hist_div_ref: NodeRef,
    input: String,
    tcl_interp: Interp<AppCtx>,
    hist: Vec<(String, Vec<Result<Value, Exception>>)>,
    current_hist_idx: Option<usize>,
}
pub enum Msg {
    None,
    UpdateInput(String),
    // RunCmd,
    KeyDown(Key),
}

pub enum Key {
    Enter,
    ArrowUp,
    ArrowDown,
}

pub fn cmd_square(interp: &mut Interp<AppCtx>, argv: &[Value]) -> MoltResult {
    // Correct number of arguments?
    check_args(1, argv, 2, 2, "x")?;
    // Get x, if it's an integer
    let x = argv[1].as_int()?;
    let out = x * x;
    interp.context.num = out as usize;
    // Return the result.
    molt_ok!(out)
}

impl App {
    fn execute(&mut self, cmd: String) {
        let out = self.tcl_interp.eval(&cmd);
        let mut outs = mem::take(&mut self.tcl_interp.std_buff);
        outs.push(out);
        self.hist.push((cmd.trim().into(), outs));
    }
}

impl Component for App {
    type Message = Msg;

    type Properties = ();

    fn create(_ctx: &Context<Self>) -> Self {
        let tcl_interp = Interp::new(
            AppCtx { num: 0 },
            gen_command!(
                AppCtx,
                // native commands
                [],
                // embedded commands
                [("square", cmd_square)]
            ),
            false,
        );

        let mut app = Self {
            hist_div_ref: NodeRef::default(),
            input_div_ref: NodeRef::default(),
            input: String::new(),
            tcl_interp,
            hist: Vec::new(),
            current_hist_idx: None,
        };
        app.execute(
            "proc say_hello {name} {
    puts \"Hello, $name!\"
}"
            .into(),
        );
        app.execute("say_hello \"World\"".into());
        app.execute(
            "set a {}
for {set i 1} {$i < 6} {incr i} {
    puts $i
    square $i
    if {$i == 4} {
        break
    }
    lappend a $i
}
set a"
                .into(),
        );
        app.execute("square \"abc\"".into());
        app.execute("info cmdtype puts".into());
        app.execute("info cmdtype square".into());
        app.execute("info cmdtype say_hello".into());
        app.execute("info commands".into());
        app
    }
    fn update(&mut self, _ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::KeyDown(key) => match key {
                Key::Enter => {
                    if let Some(element) = self.hist_div_ref.cast::<web_sys::Element>() {
                        element.set_scroll_top(element.scroll_height());
                    }
                    let cmd = mem::take(&mut self.input);
                    if !cmd.is_empty() {
                        self.execute(cmd);
                    }
                    self.current_hist_idx = None;
                    true
                }
                Key::ArrowUp => {
                    if self.input.is_empty() || self.current_hist_idx.is_some() {
                        match self.current_hist_idx.as_mut() {
                            Some(i) => {
                                if *i != 0 {
                                    *i -= 1;
                                    if let Some((hist_cmd, _)) = self.hist.get(*i) {
                                        self.input = hist_cmd.clone();
                                    }
                                    true
                                } else {
                                    false
                                }
                            }
                            None => {
                                let i = self.hist.len() - 1;
                                self.current_hist_idx = Some(i);
                                if let Some((hist_cmd, _)) = self.hist.get(i) {
                                    self.input = hist_cmd.clone();
                                }
                                true
                            }
                        }
                    } else {
                        false
                    }
                }
                Key::ArrowDown => {
                    if self.input.is_empty() || self.current_hist_idx.is_some() {
                        match self.current_hist_idx.as_mut() {
                            Some(i) => {
                                if *i != self.hist.len() - 1 {
                                    *i += 1;
                                    if let Some((hist_cmd, _)) = self.hist.get(*i) {
                                        self.input = hist_cmd.clone();
                                    }
                                    true
                                } else {
                                    false
                                }
                            }
                            None => {
                                let i = self.hist.len() - 1;
                                self.current_hist_idx = Some(i);
                                if let Some((hist_cmd, _)) = self.hist.get(i) {
                                    self.input = hist_cmd.clone();
                                }
                                true
                            }
                        }
                    } else {
                        false
                    }
                }
            },
            Msg::UpdateInput(s) => {
                self.input = s;
                self.current_hist_idx = None;
                if self.input == "\n" {
                    self.input.clear();
                }
                true
            }
            Msg::None => false,
        }
    }
    fn rendered(&mut self, _ctx: &Context<Self>, first_render: bool) {
        if first_render {
            // NOTICE: slip scroll animation
            if let Some(element) = self.hist_div_ref.cast::<web_sys::Element>() {
                let target_scroll_top = element.scroll_height();
                let current_scroll_top = element.scroll_top();
                let distance = target_scroll_top - current_scroll_top;
                let steps = 40;
                let step_duration = 25;
                for step in 0..steps {
                    let current_scroll_top = current_scroll_top + distance * step / steps;
                    let element = element.clone();
                    Timeout::new((step_duration * step) as u32, move || {
                        element.set_scroll_top(current_scroll_top);
                    })
                    .forget();
                }
            }
        }
    }
    fn view(&self, ctx: &Context<Self>) -> Html {
        html! {
          <>
          <div>
          <code>{"The context number is "}</code><code style="color:red;">{self.tcl_interp.context.num}</code><code>{", run `square [number]` to change it"}</code>
          </div>
          <ul ref={self.hist_div_ref.clone()}
          style="text-wrap:nowrap;height:50vh;margin:0px; overflow-y: auto; padding:10px;padding-inline-start:5px;padding-inline-end:5px; background: #f0f0f0;overflow-y:auto;overflow-x:auto;">
            { for self.hist.iter().map(|(cmd_ctx,outs)|{
              let mut has_err = false;
              let out_html = html!{
                {for outs.iter().enumerate().map(
                  |(i,out)|{
                    match out{
                      Ok(s) => html!(<code style="margin:0px;color:#56a2c7"> { s.to_string() }{if i==(outs.len()-1){html!()}else{html!(<br />)}}</code>),
                      Err(s) => {
                        has_err=true;
                        html!(<code style="margin:0px;color:red;"> { s.error_info().to_string() }{if i==(outs.len()-1){html!()}else{html!(<br />)}}</code>)},
                    }
                  }
                )}
              };
              let (style,icon) = if has_err{
                ("color:red;",IconId::FontAwesomeSolidXmark)
              }else{
                ("color:green;",IconId::BootstrapCheckLg)
              };
              html!{
                <li style="padding:0px;margin:0px;list-style:none;white-space:nowrap;">
                  <div style="display: flex;flex-wrap: nowrap;">
                    <Icon style={style} icon_id={icon} height={"10px".to_owned()} width={"15px".to_owned()}/>
                    <code style="white-space: pre-wrap;">
                      {cmd_ctx}
                    </code>
                  </div>
                  {out_html}
                </li>
              }
            })}
          </ul>
          <textarea
              class="terminal-input"
              ref={self.input_div_ref.clone()}
              value={self.input.clone()}
              oninput={ctx.link().callback(|e: InputEvent| {
                let input: web_sys::HtmlInputElement = e.target_unchecked_into();
                Msg::UpdateInput(input.value())
              })}
              onkeydown={ctx.link().callback(|e: KeyboardEvent| {
                match e.key().as_str(){
                  "Enter" => Msg::KeyDown(Key::Enter),
                  "ArrowUp" => Msg::KeyDown(Key::ArrowUp),
                  "ArrowDown" => Msg::KeyDown(Key::ArrowDown),
                  _ => Msg::None,
                }
              })}
              style="width: 100%;padding:0px;border:1px solid #ccc; box-sizing: border-box;"
          ></textarea>
          </>
        }
    }
}

fn main() {
    wasm_logger::init(wasm_logger::Config::new(log::Level::Trace));
    yew::Renderer::<App>::new().render();
}
