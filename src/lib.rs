#![recursion_limit = "4096"]

#[macro_use]
extern crate serde_derive;

mod markdown;

use yew::format::Json;
use yew::services::storage::Area;
use yew::services::{DialogService, StorageService};
use yew::{html, Component, ComponentLink, Html, InputData, Renderable, ShouldRender};

const KEY: &'static str = "yew.crm.database";

#[derive(Serialize, Deserialize)]
struct Database {
    clients: Vec<Client>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Client {
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

impl Client {
    fn empty() -> Self {
        Client {
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

#[derive(Debug)]
pub enum Scene {
    ClientsList,
    NewClientForm(Client),
    TFMPWIForm,
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
    UpdateFirstName(String),
    UpdateLastName(String),
    UpdateDescription(String),
    UpdateFrequency(f64),
    UpdateVelocity(f64),
    Calc_l_p,
    Clear,
}

impl Component for Model {
    type Message = Msg;
    type Properties = ();

    fn create(_: Self::Properties, link: ComponentLink<Self>) -> Self {
        let storage = StorageService::new(Area::Local);
        let Json(database) = storage.restore(KEY);
        let database = database.unwrap_or_else(|_| Database {
            clients: Vec::new(),
        });
        Model {
            link,
            storage,
            dialog: DialogService::new(),
            database,
            scene: Scene::ClientsList,
        }
    }

    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        let mut new_scene = None;
        match self.scene {
            Scene::ClientsList => match msg {
                Msg::SwitchTo(Scene::NewClientForm(client)) => {
                    new_scene = Some(Scene::NewClientForm(client));
                }
                Msg::SwitchTo(Scene::TFMPWIForm) => {
                    new_scene = Some(Scene::TFMPWIForm);
                }
                Msg::SwitchTo(Scene::Settings) => {
                    new_scene = Some(Scene::Settings);
                }
                unexpected => {
                    panic!(
                        "Unexpected message when clients list shown: {:?}",
                        unexpected
                    );
                }
            },
            Scene::NewClientForm(ref mut client) => match msg {
                Msg::UpdateFrequency(val)=> {
                    client.frequency = val;
                }
                Msg::UpdateVelocity(val)=> {
                    client.velocity = val;
                }
                /*
                Msg::UpdateFirstName(val) => {
                    println!("Input: {}", val);
                    client.first_name = val;
                }
                Msg::UpdateLastName(val) => {
                    println!("Input: {}", val);
                    client.last_name = val;
                }
                */
                Msg::UpdateDescription(val) => {
                    println!("Input: {}", val);
                    client.description = val;
                }
                Msg::Calc_l_p=> {
                    if (client.frequency == 0.0) | (client.velocity == 0.0) {
                        client.description = "频率/声速中有0值，请检查".into()
                    } else{
                        client.lambda = client.velocity / 1000.0 / client.frequency;
                        client.pitch = client.lambda / 2.0;
                        client.description = format!("波长为{}mm\npitch最小值为{}mm", client.lambda, client.pitch);
                    }
                }
                Msg::AddNew => {
                    let mut new_client = Client::empty();
                    ::std::mem::swap(client, &mut new_client);
                    self.database.clients.push(new_client);
                    self.storage.store(KEY, Json(&self.database));
                }
                Msg::SwitchTo(Scene::ClientsList) => {
                    new_scene = Some(Scene::ClientsList);
                }
                unexpected => {
                    panic!(
                        "Unexpected message during new client editing: {:?}",
                        unexpected
                    );
                }
            },
            Scene::TFMPWIForm => match msg {
                Msg::SwitchTo(Scene::ClientsList) => {
                    new_scene = Some(Scene::ClientsList);
                },
                unexpected=> {
                    panic!("Unexpected message for settings scene: {:?}", unexpected);
                }
            },
            Scene::Settings => match msg {
                Msg::Clear => {
                    let ok = { self.dialog.confirm("Do you really want to clear the data?") };
                    if ok {
                        self.database.clients.clear();
                        self.storage.remove(KEY);
                    }
                }
                Msg::SwitchTo(Scene::ClientsList) => {
                    new_scene = Some(Scene::ClientsList);
                }
                unexpected => {
                    panic!("Unexpected message for settings scene: {:?}", unexpected);
                }
            },
        }
        if let Some(new_scene) = new_scene.take() {
            self.scene = new_scene;
        }
        true
    }

    fn view(&self) -> Html {
        match self.scene {
            Scene::ClientsList => html! {
                <div class="crm">
                    <div class="clients">
                        { for self.database.clients.iter().map(Renderable::render) }
                    </div>
                    <button onclick=self.link.callback(|_| Msg::SwitchTo(Scene::NewClientForm(Client::empty())))>{ "波长&Pitch" }</button>
                    <button onclick=self.link.callback(|_| Msg::SwitchTo(Scene::TFMPWIForm))>{ "TFM PWI演示" }</button>
                    <button onclick=self.link.callback(|_| Msg::SwitchTo(Scene::Settings))>{ "Settings" }</button>
                </div>
            },
            Scene::NewClientForm(ref client) => html! {
                <div class="crm">
                    <div class="names">
                        //{ client.view_first_name_input(&self.link) }
                        //{ client.view_last_name_input(&self.link) }
                        { client.view_description_textarea(&self.link) }
                        { client.view_frequency_input(&self.link) }
                        { client.view_velocity_input(&self.link) }
                    </div>
                    <button onclick=self.link.callback(|_| Msg::Calc_l_p)>{ "计算" }</button>
                    <button //disabled=client.first_name.is_empty() || client.last_name.is_empty()
                            onclick=self.link.callback(|_| Msg::AddNew)>{ "Add New" }</button>
                    <button onclick=self.link.callback(|_| Msg::SwitchTo(Scene::ClientsList))>{ "Go Back" }</button>
                </div>
            },
            Scene::TFMPWIForm => html! {
                <div class="tfm">
                    <button>{"TFM演示"}</button>
                    <button>{"PWI演示"}</button>
                    <button onclick=self.link.callback(|_| Msg::SwitchTo(Scene::ClientsList))>{ "返回" }</button>
                </div>
            },
            Scene::Settings => html! {
                <div>
                    <button onclick=self.link.callback(|_| Msg::Clear)>{ "Clear Database" }</button>
                    <button onclick=self.link.callback(|_| Msg::SwitchTo(Scene::ClientsList))>{ "Go Back" }</button>
                </div>
            },
        }
    }
}

impl Renderable for Client {
    fn render(&self) -> Html {
        html! {
            <div class="client">
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

impl Client {
    fn view_frequency_input(&self, link: &ComponentLink<Model>) -> Html {
        html! {
            <input class="new-client frequency"
                   placeholder="频率"
                   value=&self.frequency
                   oninput=link.callback(|e: InputData| Msg::UpdateFrequency(e.value.parse().unwrap())) />
        }
    }
    fn view_velocity_input(&self, link: &ComponentLink<Model>) -> Html {
        html! {
            <input class="new-client velocity"
                   placeholder="声速"
                   value=&self.velocity
                   oninput=link.callback(|e: InputData| Msg::UpdateVelocity(e.value.parse().unwrap())) />
        }
    }
    /*
    fn view_first_name_input(&self, link: &ComponentLink<Model>) -> Html {
        html! {
            <input class="new-client firstname"
                   placeholder="First name"
                   value=&self.first_name
                   oninput=link.callback(|e: InputData| Msg::UpdateFirstName(e.value)) />
        }
    }

    fn view_last_name_input(&self, link: &ComponentLink<Model>) -> Html {
        html! {
            <input class="new-client lastname"
                   placeholder="Last name"
                   value=&self.last_name
                   oninput=link.callback(|e: InputData| Msg::UpdateLastName(e.value)) />
        }
    }
    */
    fn view_description_textarea(&self, link: &ComponentLink<Model>) -> Html {
        html! {
            <textarea class=("new-client", "description")
               placeholder="Description"
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
                    { client.view_vel_input(&self.link)}
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