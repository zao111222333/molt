use gloo::timers::callback::Timeout;
// re-export molt_forked
use molt::prelude::*;
pub use molt_forked as molt;
use std::{mem, rc::Rc};
use web_sys::HtmlTextAreaElement;
use yew::prelude::*;
use yew_icons::{Icon, IconId};

pub struct Terminal {
    input_div_ref: NodeRef,
    hist_div_ref: NodeRef,
    input: String,
    input_tmp: String,
    current_hist_idx: Option<usize>,
}

#[derive(Debug, Properties, PartialEq)]
pub struct TerminalProp {
    pub hist: Rc<Vec<(String, Vec<Result<Value, Exception>>)>>,
    pub on_run_cmd: Callback<String>,
}

pub enum TerminalMsg {
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

impl Terminal {
    fn input_div_cursor_to_end(&mut self) {
        if let Some(textarea) = self.input_div_ref.cast::<HtmlTextAreaElement>() {
            let length = self.input.chars().count() as u32;
            Timeout::new(5, move || {
                _ = textarea.set_selection_range(length, length);
            })
            .forget();
        }
    }
}

impl Component for Terminal {
    type Message = TerminalMsg;
    type Properties = TerminalProp;

    fn create(_ctx: &Context<Self>) -> Self {
        Self {
            hist_div_ref: NodeRef::default(),
            input_div_ref: NodeRef::default(),
            input: String::new(),
            input_tmp: String::new(),
            current_hist_idx: None,
        }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            TerminalMsg::KeyDown(key) => match key {
                Key::Enter => {
                    let cmd = mem::take(&mut self.input);
                    if !cmd.is_empty() {
                        ctx.props().on_run_cmd.emit(cmd);
                    }
                    if let Some(element) = self.hist_div_ref.cast::<web_sys::Element>() {
                        Timeout::new(5 as u32, move || {
                            element.set_scroll_top(element.scroll_height());
                        })
                        .forget();
                    }
                    self.current_hist_idx = None;
                    self.input_tmp.clear();
                    true
                }
                Key::ArrowUp => match self.current_hist_idx.as_mut() {
                    Some(0) => false,
                    Some(i) => {
                        if *i == ctx.props().hist.len() {
                            self.input_tmp = mem::take(&mut self.input);
                        }
                        *i -= 1;
                        if let Some((hist_cmd, _)) = ctx.props().hist.get(*i) {
                            self.input = hist_cmd.clone();
                        }
                        self.input_div_cursor_to_end();
                        true
                    }
                    None => {
                        let i = ctx.props().hist.len() - 1;
                        self.current_hist_idx = Some(i);
                        if let Some((hist_cmd, _)) = ctx.props().hist.get(i) {
                            self.input_tmp = mem::take(&mut self.input);
                            self.input = hist_cmd.clone();
                        }
                        self.input_div_cursor_to_end();
                        true
                    }
                },
                Key::ArrowDown => match self.current_hist_idx.as_mut() {
                    Some(i) => {
                        if *i == ctx.props().hist.len() {
                            false
                        } else if *i == ctx.props().hist.len() - 1 {
                            *i += 1;
                            self.input = mem::take(&mut self.input_tmp);
                            true
                        } else {
                            *i += 1;
                            if let Some((hist_cmd, _)) = ctx.props().hist.get(*i) {
                                self.input = hist_cmd.clone();
                            }
                            true
                        }
                    }
                    None => false,
                },
            },
            TerminalMsg::UpdateInput(s) => {
                self.input = s;
                self.current_hist_idx = None;
                self.input_tmp.clear();
                if self.input == "\n" {
                    self.input.clear();
                }
                true
            }
            TerminalMsg::None => false,
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
                let step_duration = 20;
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
          <ul ref={self.hist_div_ref.clone()}
          style="text-wrap:nowrap;height:50vh;margin:0px; overflow-y: auto; padding:10px;padding-inline-start:5px;padding-inline-end:5px; background: #f0f0f0;overflow-y:auto;overflow-x:auto;">
            { for ctx.props().hist.iter().map(|(cmd_ctx,outs)|{
              let mut has_err = false;
              let out_html = html!{
                {for outs.iter().enumerate().map(
                  |(i,out)|{
                    match out{
                      Ok(s) => html!(<code style="margin:0px;color:#56a2c7;white-space: pre-wrap;"> { s.to_string() }{if i==(outs.len()-1){html!()}else{html!(<br />)}}</code>),
                      Err(s) => {
                        has_err=true;
                        html!(<code style="margin:0px;color:red;white-space: pre-wrap;"> { s.error_info().to_string() }{if i==(outs.len()-1){html!()}else{html!(<br />)}}</code>)},
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
                  <div style="padding-left:15px">
                    {out_html}
                  </div>
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
                TerminalMsg::UpdateInput(input.value())
              })}
              onkeydown={ctx.link().callback(|e: KeyboardEvent| {
                match e.key().as_str(){
                  "Enter" => TerminalMsg::KeyDown(Key::Enter),
                  "ArrowUp" => TerminalMsg::KeyDown(Key::ArrowUp),
                  "ArrowDown" => TerminalMsg::KeyDown(Key::ArrowDown),
                  _ => TerminalMsg::None,
                }
              })}
              style="width: 100%;padding:0px;border:1px solid #ccc; box-sizing: border-box;"
          ></textarea>
          </>
        }
    }
}
