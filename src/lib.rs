#![recursion_limit = "4096"]

#[macro_use]
extern crate serde_derive;

mod markdown;

use yew::format::Json;
use yew::services::storage::Area;
use yew::services::{DialogService, StorageService};
use yew::{html, Component, ComponentLink, Html, InputData, Renderable, ShouldRender};
use std::env;
use std::fs::File;
use std::io::prelude::*;
use crate::Scene::SceneList;

const KEY: &'static str = "yew.crm.database";

#[derive(Serialize, Deserialize)]
struct Database {
    probes: Vec<Probe>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Probe {
    //first_name: String,
    //last_name: String,
    description: String,
    //存储输入数据的变量
    frequency: f64,
    velocity: f64,
    //存储结果的变量
    lambda: f64,
    pitch: f64,
}

impl Probe {
    fn empty() -> Self {
        Probe {
            //first_name: "".into(),
            //last_name: "".into(),
            description: "".into(),
            frequency: 0.0,
            velocity: 0.0,
            lambda: 0.0,
            pitch: 0.0,
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct BeamAngle {
    incidence_min: f64,
    refraction_min: f64,

    incidence_max: f64,
    refraction_max: f64,

    incidence_min_steel: f64,
    incidence_max_steel: f64,

    velocity_steel: f64,
    velocity_medium: f64,
    //reflection: f64,
}

impl BeamAngle {
    fn empty() -> Self {
        BeamAngle {
            incidence_min: 0.0,
            refraction_min: 0.0,

            incidence_max: 0.0,
            refraction_max: 0.0,

            incidence_min_steel: 0.0,
            incidence_max_steel: 0.0,

            velocity_steel: 0.0,
            velocity_medium: 0.0,
            //reflection: f64,
        }
    }

    fn incidence_min_input(&self, link: &ComponentLink<Model>) -> Html {
        html!{
        <input class="beam-angle"
                   placeholder="入射角（小）"
                   //不更新内容的话会怎么样？
                   //value=&self.frequency
                   oninput=link.callback(|e: InputData| Msg::UpdateIncidenceAngleMin(e.value.parse().unwrap())) />
        }
    }
}


#[derive(Debug)]
pub enum Scene {
    SceneList,
    NewProbeForm(Probe),
    TFMPWIForm,
    RefractionAngle,
    Settings,
}

pub struct Model {
    link: ComponentLink<Self>,
    storage: StorageService,
    dialog: DialogService,
    database: Database,
    scene: Scene,
}

#[derive(Debug)]
pub enum Msg {
    SwitchTo(Scene),
    AddNew,
    //UpdateFirstName(String),
    //UpdateLastName(String),
    UpdateDescription(String),
    UpdateFrequency(f64),
    UpdateVelocity(f64),
    UpdateIncidenceAngleMin(f64),
    CalcLP,
    Clear,
}

impl Component for Model {
    type Message = Msg;
    type Properties = ();

    fn create(_: Self::Properties, link: ComponentLink<Self>) -> Self {
        let storage = StorageService::new(Area::Local);
        let Json(database) = storage.restore(KEY);
        let database = database.unwrap_or_else(|_| Database {
            probes: Vec::new(),
        });
        Model {
            link,
            storage,
            dialog: DialogService::new(),
            database,
            scene: Scene::SceneList,
        }
    }

    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        let mut new_scene = None;
        match self.scene {
            Scene::SceneList => match msg {
                Msg::SwitchTo(Scene::NewProbeForm(probe)) => {
                    new_scene = Some(Scene::NewProbeForm(probe));
                }
                Msg::SwitchTo(Scene::TFMPWIForm) => {
                    new_scene = Some(Scene::TFMPWIForm);
                }
                Msg::SwitchTo(Scene::Settings) => {
                    new_scene = Some(Scene::Settings);
                }
                unexpected => {
                    panic!(
                        "Unexpected message when probes list shown: {:?}",
                        unexpected
                    );
                }
            },
            Scene::NewProbeForm(ref mut probe) => match msg {
                Msg::UpdateFrequency(val)=> {
                    probe.frequency = val;
                }
                Msg::UpdateVelocity(val)=> {
                    probe.velocity = val;
                }
                /*
                Msg::UpdateFirstName(val) => {
                    println!("Input: {}", val);
                    probe.first_name = val;
                }
                Msg::UpdateLastName(val) => {
                    println!("Input: {}", val);
                    probe.last_name = val;
                }
                */
                Msg::UpdateDescription(val) => {
                    println!("Input: {}", val);
                    probe.description = val;
                }
                Msg::CalcLP => {
                    if (probe.frequency == 0.0) | (probe.velocity == 0.0) {
                        probe.description = "频率/声速中有0值，请检查".into()
                    } else {
                        probe.lambda = probe.velocity / 1000.0 / probe.frequency;
                        probe.pitch = probe.lambda / 2.0;
                        probe.description = format!("波长为{}mm\npitch最小值为{}mm", probe.lambda, probe.pitch);
                    }
                }
                Msg::CalcLP => {
                    //TBD
                }
                Msg::AddNew => {
                    let mut new_probe = Probe::empty();
                    ::std::mem::swap(probe, &mut new_probe);
                    self.database.probes.push(new_probe);
                    self.storage.store(KEY, Json(&self.database));
                }
                Msg::SwitchTo(Scene::SceneList) => {
                    new_scene = Some(Scene::SceneList);
                }
                unexpected => {
                    panic!(
                        "Unexpected message during new probe editing: {:?}",
                        unexpected
                    );
                    //错误处理需要更人性化
                }
            },
            Scene::TFMPWIForm => match msg {
                Msg::SwitchTo(Scene::SceneList) => {
                    new_scene = Some(Scene::SceneList);
                },
                unexpected => {
                    panic!("Unexpected message for settings scene: {:?}", unexpected);
                },
                //错误处理方式需改进
            },
            Scene::RefractionAngle => match msg {
                Msg::SwitchTo(Scene::SceneList) => {
                    new_scene = Some(Scene::SceneList);
                },
                unexpected => {
                    panic!("未知参数，折射角计算模块{:?}", unexpected);
                },
            },
            Scene::Settings => match msg {
                Msg::Clear => {
                    let ok = { self.dialog.confirm("确实要清除数据吗?") };
                    if ok {
                        self.database.probes.clear();
                        self.storage.remove(KEY);
                    }
                }
                Msg::SwitchTo(Scene::SceneList) => {
                    new_scene = Some(Scene::SceneList);
                }
                unexpected => {
                    panic!("Unexpected message for settings scene: {:?}", unexpected);
                }
                //错误处理方式需改进
            },
        }
        if let Some(new_scene) = new_scene.take() {
            self.scene = new_scene;
        }
        true
    }

    fn view(&self) -> Html {
        match self.scene {
            Scene::SceneList => html! {
                <div class="crm">
                    <div class="probes">
                        { for self.database.probes.iter().map(Renderable::render) }
                    </div>
                    <button onclick=self.link.callback(|_| Msg::SwitchTo(Scene::NewProbeForm(Probe::empty())))>{ "波长&Pitch" }</button>
                    <button onclick=self.link.callback(|_| Msg::SwitchTo(Scene::TFMPWIForm))>{ "TFM PWI演示" }</button>
                    <button onclick=self.link.callback(|_| Msg::SwitchTo(Scene::Settings))>{ "Settings" }</button>
                </div>
            },
            Scene::NewProbeForm(ref probe) => html! {
                <div class="crm">
                    <div class="names">
                        //{ probe.view_first_name_input(&self.link) }
                        //{ probe.view_last_name_input(&self.link) }
                        { probe.view_description_textarea(&self.link) }
                        { probe.view_frequency_input(&self.link) }
                        { probe.view_velocity_input(&self.link) }
                    </div>
                    <button onclick=self.link.callback(|_| Msg::CalcLP)>{ "计算" }</button>
                    <button //disabled=probe.first_name.is_empty() || probe.last_name.is_empty()
                            onclick=self.link.callback(|_| Msg::AddNew)>{ "保存" }</button>
                    <button onclick=self.link.callback(|_| Msg::SwitchTo(Scene::SceneList))>{ "返回" }</button>
                </div>
            },
            Scene::TFMPWIForm => html! {
                <div class="tfm">
                    <button>{"TFM演示"}</button>
                    <button>{"PWI演示"}</button>
                    <a href="https://eddyfi.com/academy.html">
                    <button>{"TFM线上学习课程"}</button>
                    </a>
                    <button onclick=self.link.callback(|_| Msg::SwitchTo(Scene::SceneList))>{ "返回" }</button>
                    <hr/>
                    <img src="Acquisition-FMC-ET-01.gif"  alt="TFM数据采集" title="TFM数据采集FMC"/>
                    <img src="RECONSTRUCTION-TFM-ET.gif"  alt="TFM数据重建" title="TFM数据重建"/>
                    <hr/>
                    //<img src="Acquisition-FMC-ET-01.gif"  alt="TFM数据采集" title="TFM数据重建"/>
                    //视频播放,替换
                    //目前视频播放仅支持mp4, webm和ogg格式
                    <video src="N600_HVAC_HEATEXCHANGER_ECTINSPECTION_SUBTITLEMASTER_w(2)_480.mp4" controls=true />
                    //<img  dynsrc="file:///D:/Rust/webTools/img/N600_HVAC_HEATEXCHANGER_ECTINSPECTION_SUBTITLEMASTER_w(2)_480.mp4"  start="mouseover" alt="PWI激发"/>
                </div>
            },
            Scene::RefractionAngle => html! {
                <div class="refraction">
                    <button onclick=self.link.callback(|_| Msg::SwitchTo(Scene::SceneList))>{ "返回" }</button>
                </div>
            },
            Scene::Settings => html! {
                <div>
                    <button onclick=self.link.callback(|_| Msg::Clear)>{ "清除所有数据" }</button>
                    <button onclick=self.link.callback(|_| Msg::SwitchTo(Scene::SceneList))>{ "返回" }</button>
                </div>
            },
        }
    }
}

impl Renderable for Probe {
    fn render(&self) -> Html {
        html! {
            <div class="probe">
                //<p>{ format!("First Name: {}", self.first_name) }</p>
                //<p>{ format!("Last Name: {}", self.last_name) }</p>
                <p>{ format!("Frequency: {}", self.frequency) }</p>
                <p>{ format!("Velocity: {}", self.velocity) }</p>
                <p>{ format!("Lambda: {}", self.lambda) }</p>
                <p>{ format!("Pitch: {}", self.pitch) }</p>
                <p>{ "Description:" }</p>
                { markdown::render_markdown(&self.description) }
            </div>
        }
    }
}

impl Probe {
    fn view_frequency_input(&self, link: &ComponentLink<Model>) -> Html {
        html! {
            <input class="new-probe"
                   placeholder="频率"
                   //不更新内容的话会怎么样？
                   //value=&self.frequency
                   oninput=link.callback(|e: InputData| Msg::UpdateFrequency(e.value.parse().unwrap())) />
        }
    }
    fn view_velocity_input(&self, link: &ComponentLink<Model>) -> Html {
        html! {
            <input class="new-probe"
                   placeholder="声速"
                   //先注释掉更新内容这一行，目前看来可以正常计算
                   //value=&self.velocity
                   oninput=link.callback(|e: InputData| Msg::UpdateVelocity(e.value.parse().unwrap())) />
        }
    }
    /*
    fn view_first_name_input(&self, link: &ComponentLink<Model>) -> Html {
        html! {
            <input class="new-probe firstname"
                   placeholder="First name"
                   value=&self.first_name
                   oninput=link.callback(|e: InputData| Msg::UpdateFirstName(e.value)) />
        }
    }

    fn view_last_name_input(&self, link: &ComponentLink<Model>) -> Html {
        html! {
            <input class="new-probe lastname"
                   placeholder="Last name"
                   value=&self.last_name
                   oninput=link.callback(|e: InputData| Msg::UpdateLastName(e.value)) />
        }
    }
    */
    fn view_description_textarea(&self, link: &ComponentLink<Model>) -> Html {
        html! {
            <textarea class=("new-probe", "description")
               placeholder="结果"
               value=&self.description
               oninput=link.callback(|e: InputData| Msg::UpdateDescription(e.value)) />
        }
    }
}

/*
#![recursion_limit = "4096"]

use yew::{html, Component, ComponentLink, Html, InputData, ShouldRender};

pub struct Model {
    link: ComponentLink<Self>,
    //value: String,
    velocity: f64,
    vel: f64,
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
                    { probe.view_vel_input(&self.link)}
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
*/
