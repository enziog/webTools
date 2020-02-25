#![recursion_limit = "4096"]

use yew::{html, Component, ComponentLink, Html, InputData, ShouldRender};

pub struct Model {
    link: ComponentLink<Self>,
    //value: String,
    velocity: f64,
    frequency: f64,
    //存储结果
    lambda: f64,
}

pub enum Msg {
    GotVelocity(String),
    GotFrequency(String),
    Clicked,
}

impl Component for Model {
    type Message = Msg;
    type Properties = ();

    fn create(_: Self::Properties, link: ComponentLink<Self>) -> Self {
        Model {
            link,
            velocity: 0.0,
            frequency: 0.0,
            //结果存储
            lambda: 0.0,
        }
    }

    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        match msg {
            Msg::GotFrequency(new_value) => {
                self.frequency = new_value.parse().unwrap();
            }
            Msg::GotVelocity(new_value) => {
                self.velocity = new_value.parse().unwrap();
            }
            Msg::Clicked => {
                self.lambda = calc(self.frequency, self.velocity);
                //println!("波长为： {}mm", self.lambda);
            }
        }
        true
    }

    fn view(&self) -> Html {
        html! {
            <div>
                <div>
                    <textarea rows=1
                        value=&self.velocity
                        oninput=self.link.callback(|f: InputData| Msg::GotVelocity(f.value))
                        placeholder="声速">
                    </textarea>
                    <textarea rows=1
                        value=&self.frequency
                        oninput=self.link.callback(|e: InputData| Msg::GotFrequency(e.value))
                        placeholder="频率">
                    </textarea>
                    <button onclick=self.link.callback(|_| Msg::Clicked)>{ "计算" }</button>
                </div>
                <div>
                    {&self.lambda}
                </div>
            </div>
        }
    }
}

fn calc(freq: f64, velo: f64) -> f64 {
    let lambda = velo / 1000.0 / freq;
    lambda
}