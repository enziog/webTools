#![recursion_limit = "4096"]

#[macro_use]
extern crate serde_derive;

mod markdown;

use crate::Scene::SceneList;
use std::env;
use std::fs::File;
use std::io::prelude::*;
use yew::format::Json;
use yew::services::storage::Area;
use yew::services::{DialogService, StorageService};
use yew::{html, Component, ComponentLink, Html, InputData, Renderable, ShouldRender};

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

    refraction_steel_min: f64,
    refraction_steel_max: f64,

    velocity_steel: f64,
    velocity_incidence: f64,
    velocity_medium: f64,

    result: String,
    //reflection: f64,
}

impl BeamAngle {
    fn empty() -> Self {
        BeamAngle {
            incidence_min: 0.0,
            refraction_min: 0.0,

            incidence_max: 0.0,
            refraction_max: 0.0,

            refraction_steel_min: 0.0,
            refraction_steel_max: 0.0,

            velocity_steel: 0.0,
            velocity_incidence: 0.0,
            velocity_medium: 0.0,
            //reflection: f64,
            result: "".into(),
        }
    }

    fn incidence_min_input(&self, link: &ComponentLink<Model>) -> Html {
        html! {
        <input class="beam-angle"
                   placeholder="入射角（小）"
                   oninput=link.callback(|e: InputData| Msg::UpdateIncidenceAngleMin(e.value.parse().unwrap())) />
        }
    }
    fn incidence_max_input(&self, link: &ComponentLink<Model>) -> Html {
        html! {
        <input class="beam-angle"
                   placeholder="入射角（大）"
                   oninput=link.callback(|e: InputData| Msg::UpdateIncidenceAngleMax(e.value.parse().unwrap())) />
        }
    }
    fn velocity_incidence_input(&self, link: &ComponentLink<Model>) -> Html {
        html! {
        <input class="beam-angle"
                   placeholder="介质声速（入射角）"
                   oninput=link.callback(|e: InputData| Msg::UpdateVelocityIncidence(e.value.parse().unwrap())) />
        }
    }
    fn velocity_refraction_input(&self, link: &ComponentLink<Model>) -> Html {
        html! {
        <input class="beam-angle"
                   placeholder="介质声速（折射角）"
                   oninput=link.callback(|e: InputData| Msg::UpdateVelocityRefraction(e.value.parse().unwrap())) />
        }
    }
    fn view_result(&self, link: &ComponentLink<Model>) -> Html {
        html! {
            <textarea class=("beam-angle", "result")
               placeholder="结果"
               value=&self.result
               oninput=link.callback(|e: InputData| Msg::UpdateDescription(e.value)) />
        }
    }
}

//这段代码的用途是什么？？？
impl Renderable for BeamAngle {
    fn render(&self) -> Html {
        html! {
            <div class="beam-angle">
                <p>{ format!("Incidence Min: {}", self.incidence_min) }</p>
                <p>{ format!("Incidence Max: {}", self.incidence_max) }</p>
                <p>{ format!("Incidence in Steel Min: {}", self.refraction_steel_min) }</p>
                <p>{ format!("Incidence in Steel Max: {}", self.refraction_steel_max) }</p>
                <p>{ "Result:" }</p>
                { markdown::render_markdown(&self.result) }
            </div>
        }
    }
}

#[derive(Debug)]
pub enum Scene {
    SceneList,
    ProbeForm(Probe),
    //BeamAngleForm(BeamAngle),
    TFMPWIForm,
    RefractionAngle(BeamAngle),
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
    //探头计算
    //UpdateFirstName(String),
    //UpdateLastName(String),
    UpdateDescription(String),
    UpdateFrequency(f64),
    UpdateVelocity(f64),
    CalcLP,
    //折射角计算
    UpdateIncidenceAngleMin(f64),
    //u32?
    UpdateIncidenceAngleMax(f64),
    //u32?
    UpdateVelocityIncidence(f64),
    //u32?
    UpdateVelocityRefraction(f64),
    //u32?
    UpdateVelocitySteel(f64),
    //u32
    CalcRefraction,
    //
    Clear,
}

impl Component for Model {
    type Message = Msg;
    type Properties = ();

    fn create(_: Self::Properties, link: ComponentLink<Self>) -> Self {
        let storage = StorageService::new(Area::Local);
        let Json(database) = storage.restore(KEY);
        let database = database.unwrap_or_else(|_| Database { probes: Vec::new() });
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
                Msg::SwitchTo(Scene::ProbeForm(probe)) => {
                    new_scene = Some(Scene::ProbeForm(probe));
                }
                Msg::SwitchTo(Scene::RefractionAngle(beam_abngle)) => {
                    new_scene = Some(Scene::RefractionAngle(beam_abngle));
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
            Scene::ProbeForm(ref mut probe) => match msg {
                Msg::UpdateFrequency(val) => {
                    probe.frequency = val;
                }
                Msg::UpdateVelocity(val) => {
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
                        probe.description =
                            format!("波长为{}mm\npitch最小值为{}mm", probe.lambda, probe.pitch);
                    }
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
                }
                unexpected => {
                    panic!("Unexpected message for settings scene: {:?}", unexpected);
                }
                //错误处理方式需改进
            },
            Scene::RefractionAngle(ref mut beam_angle) => match msg {
                Msg::SwitchTo(Scene::SceneList) => {
                    new_scene = Some(Scene::SceneList);
                }
                Msg::UpdateIncidenceAngleMin(val) => {
                    beam_angle.incidence_min = val;
                }
                Msg::UpdateIncidenceAngleMax(val) => {
                    beam_angle.incidence_max = val;
                }
                Msg::UpdateVelocityIncidence(val) => {
                    beam_angle.velocity_incidence = val;
                }
                Msg::UpdateVelocityRefraction(val) => {
                    beam_angle.velocity_medium = val;
                }
                /*
                Msg::UpdateRefractionSteelMin(val) => {
                    beam_angle.refraction_steel_min = val;
                }
                Msg::UpdateRefractionSteelMax(val) => {
                    beam_angle.refraction_steel_max = val;
                }
                */
                Msg::CalcRefraction => {
                    //还未计算角度
                    beam_angle.refraction_min = beam_angle.incidence_min * 1.0;
                    beam_angle.refraction_max = beam_angle.incidence_max * 1.0;
                    beam_angle.result = format!(
                        "折射角范围为{}度～{}度\n按入射声速{}m/s折射声速{}m/s计算",
                        beam_angle.refraction_min, beam_angle.refraction_max, beam_angle.velocity_incidence, beam_angle.velocity_medium
                    );
                }
                unexpected => {
                    panic!("未知参数，折射角计算模块{:?}", unexpected);
                }
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
                } //错误处理方式需改进
            },
        }
        if let Some(new_scene) = new_scene.take() {
            self.scene = new_scene;
        }
        true
    }

    fn view(&self) -> Html {
        /*
        let mut html_content = "<div class="crm">
                    <div class="probes">
                        { for self.database.probes.iter().map(Renderable::render) }
                    </div>
                    <button onclick=self.link.callback(|_| Msg::SwitchTo(Scene::ProbeForm(Probe::empty())))>{ "波长&Pitch" }</button>
                    <button onclick=self.link.callback(|_| Msg::SwitchTo(Scene::TFMPWIForm))>{ "TFM PWI演示" }</button>
                    <button onclick=self.link.callback(|_| Msg::SwitchTo(Scene::Settings))>{ "Settings" }</button>
                </div>";
        */
        match self.scene {
            Scene::SceneList => html! {
                <div class="crm">
                    <div class="probes">
                        { for self.database.probes.iter().map(Renderable::render) }
                    </div>
                    <button onclick=self.link.callback(|_| Msg::SwitchTo(Scene::ProbeForm(Probe::empty())))>{ "波长&Pitch" }</button>
                    <button onclick=self.link.callback(|_| Msg::SwitchTo(Scene::RefractionAngle(BeamAngle::empty())))>{ "PA探头折射角" }</button>
                    <button onclick=self.link.callback(|_| Msg::SwitchTo(Scene::TFMPWIForm))>{ "TFM PWI演示" }</button>
                    <button onclick=self.link.callback(|_| Msg::SwitchTo(Scene::Settings))>{ "Settings" }</button>
                </div>
            },
            Scene::ProbeForm(ref probe) => html! {
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
                    <button>{"FMC-TFM演示"}</button>
                    <button>{"PWI-TFM演示"}</button>
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
            Scene::RefractionAngle(ref beam_angle) => html! {
                <div class="refraction">
                    { beam_angle.incidence_min_input(&self.link) }
                    { beam_angle.incidence_max_input(&self.link) }
                    { beam_angle.velocity_incidence_input(&self.link)}
                    { beam_angle.velocity_refraction_input(&self.link) }
                    <hr/>
                    <button onclick=self.link.callback(|_| Msg::CalcRefraction)>{"计算折射角"}</button>
                    <button onclick=self.link.callback(|_| Msg::SwitchTo(Scene::SceneList))>{ "返回" }</button>
                    <hr/>
                    { beam_angle.view_result(&self.link)}
                </div>
            },
            Scene::Settings => html! {
                <div>
                    <button onclick=self.link.callback(|_| Msg::Clear)>{ "清除所有数据" }</button>
                    <button onclick=self.link.callback(|_| Msg::SwitchTo(Scene::SceneList))>{ "返回" }</button>
                    <hr/>
                    <a href="mailto:enzio.g@qq.com">{"技术支持（邮箱）"}</a>
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
