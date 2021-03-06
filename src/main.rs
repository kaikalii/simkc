use std::{fmt, sync::Arc};

use lazy_static::lazy_static;
use rand::{thread_rng, Rng};
use yew::{html, html_impl, prelude::*, services::ConsoleService};

#[derive(Debug, Clone, Copy)]
enum Person {
    Savannah,
    Isaac,
    Miguel,
    Kai,
    Carl,
    Guest,
}

impl fmt::Display for Person {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

#[derive(Clone)]
enum State {
    Home,
    Trivia(Trivia),
    ChoosePerson,
    Unique(Person),
    Drink { correct: bool, count: usize },
    GiveDrinks(usize),
    Generic(Arc<Box<Fn() -> Html<Model> + Sync + Send>>),
}

impl From<usize> for State {
    fn from(count: usize) -> Self {
        State::Drink {
            correct: count <= 1,
            count,
        }
    }
}

impl State {
    fn generic<F>(f: F) -> State
    where
        F: Fn() -> Html<Model> + Sync + Send + 'static,
    {
        State::Generic(Arc::new(Box::new(f)))
    }
}

enum Msg {
    SetState(State),
    Entry(String),
}

#[derive(Clone)]
enum Trivia {
    MultipleChoice {
        question: String,
        choices: Vec<(String, State)>,
    },
    ShortAnswer {
        question: String,
        validate: Arc<Box<Fn(&str) -> State + Send + Sync>>,
    },
}

lazy_static! {
    static ref TRIVIA: Vec<Trivia> = vec![
        Trivia::multiple_choice(
            "Which Evan is \"Black Evan\"?",
            vec![("Evan Holmes", 4), ("Evan Hoerl", 1)],
        ),
        Trivia::multiple_choice(
            "What was Waffles' very first transformation?",
            vec![
                ("Giant Toad", 3),
                ("Snake", 4),
                ("Bear", 1),
                ("Swarm of Fleas", 4),
                ("Velociraptor", 5)
            ],
        ),
        Trivia::multiple_choice(
            "Who came up with the name \"Savisaac Migkairl\"?",
            vec![
                (Person::Carl, 3),
                (Person::Isaac, 3),
                (Person::Kai, 1),
                (Person::Miguel, 3),
                (Person::Savannah, 3),
            ],
        ),
        Trivia::short_answer(
            "Savannah has a mountain tattoo which is very similar to that of who?",
            |s| if s.trim().to_lowercase() == "grace" {
                1
            } else {
                3
            }
        ),
        Trivia::short_answer(
            "What is the significance of the phrase \"Santa Clara Java Virtual Machine\"?",
            |s| {
                let s = s.to_lowercase();
                if s.contains("name") || s.contains("last") {
                    State::GiveDrinks(4)
                } else {
                    2.into()
                }
            }
        ),
        Trivia::short_answer("Who did Jesus fail to woo?", |s| {
            let s = s.to_lowercase();
            if s.contains("frulam") || s.contains("mondath") {
                State::GiveDrinks(3)
            } else {
                3.into()
            }
        }),
        Trivia::short_answer(
            "Which stupid senior design project beat Isaac, Miguel, Carl, and Evan?",
            |s| {
                let s = s.to_lowercase();
                if s.contains("human") || s.contains("keyboard") {
                    State::GiveDrinks(3)
                } else {
                    3.into()
                }
            }
        ),
        Trivia::multiple_choice("Who is completely and uterly alone?", {
            let kai_state = State::generic(|| {
                html! {
                    <div class="question font",>{"Give Kai 5 drinks to help dull his need for companionship."}</div>
                    <div class="center",>
                        <button class="button home", type="button", onclick=|_| Msg::SetState(State::Home),>{"Home"}</button>
                    </div>
                }
            });
            vec![("Kai", kai_state); 6]
        })
    ];
}

impl Trivia {
    fn random() -> Trivia {
        let i = thread_rng().gen_range(0, TRIVIA.len());
        TRIVIA[i].clone()
    }
    fn multiple_choice<Q, C, A, S>(question: Q, choices: C) -> Trivia
    where
        Q: ToString,
        C: IntoIterator<Item = (A, S)>,
        A: ToString,
        S: Into<State>,
    {
        Trivia::MultipleChoice {
            question: question.to_string(),
            choices: choices
                .into_iter()
                .map(|(s, i)| (s.to_string(), i.into()))
                .collect(),
        }
    }
    fn short_answer<Q, V, S>(question: Q, validate: V) -> Trivia
    where
        Q: ToString,
        V: Fn(&str) -> S + Send + Sync + 'static,
        S: Into<State>,
    {
        Trivia::ShortAnswer {
            question: question.to_string(),
            validate: Arc::new(Box::new(move |s: &str| validate(s).into())),
        }
    }
    fn html(&self, model: &Model) -> Html<Model> {
        use Trivia::*;
        match self.clone() {
            MultipleChoice { question, choices } => {
                html! {
                    <div class="question font center",>
                        {question}
                    </div>
                    {
                        for choices.into_iter().map(|(answer, state)| {
                            html!{
                                <div class="center",>
                                    <button class="button", type="button", onclick=|_| Msg::SetState(state.clone()),>
                                        {answer}
                                    </button>
                                </div>
                            }
                        })
                    }
                    <div class="center",>
                        <button class="button answered",type="button", onclick=|_|
                            Msg::SetState(State::Trivia(Trivia::random())),>
                            {"This question was already answered"}
                        </button>

                        <button class="button home", type="button", onclick=|_| Msg::SetState(State::Home),>
                            {"Home"}
                        </button>
                    </div>
                }
            }
            ShortAnswer { question, validate } => {
                let entry = model.entry.clone();
                let submission = Arc::new(Box::new(move || Msg::SetState({ validate(&entry) })));
                let submission2 = submission.clone();
                html! {
                    <div class="question font",>
                        {question}
                    </div>

                    <form onsubmit=|_| submission(),>
                        <input class="entry", type="text", oninput=|s| Msg::Entry(s.value),></input>
                    </form>

                    <button class="button", type="button", onclick=|_| submission2(),>
                        {"Submit"}
                    </button>

                    <button class="button answered", type="button", onclick=|_|
                        Msg::SetState(State::Trivia(Trivia::random())),>
                        {"This question was already answered"}
                    </button>

                    <button class="button home", type="button", onclick=|_| Msg::SetState(State::Home),>
                        {"Home"}
                    </button>
                }
            }
        }
    }
}

struct Model {
    state: State,
    entry: String,
    #[allow(dead_code)]
    console: ConsoleService,
}

impl Component for Model {
    type Message = Msg;
    type Properties = ();
    fn create(_: Self::Properties, _: ComponentLink<Self>) -> Self {
        Model {
            state: State::Home,
            entry: String::new(),
            console: ConsoleService::new(),
        }
    }
    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        use Msg::*;
        match msg {
            SetState(state) => {
                self.state = state;
            }
            Entry(s) => {
                self.entry = s;
            }
        };
        true
    }
}

impl Renderable<Model> for Model {
    fn view(&self) -> Html<Self> {
        match &self.state {
            State::Generic(f) => f(),
            State::Home => html! {
                <div class="center",>
                    <h1 class="title font",>{"Savisaac Migkairl!"}</h1>
                </div>
                <div class="center",>
                    <button type="button", class="button", onclick=|_| Msg::SetState(State::Trivia(Trivia::random())),>{"Trivia"}</button>
                </div>
                <div class="center",>
                    <button type="button", class="button", onclick=|_| Msg::SetState(State::ChoosePerson),>{"Unique"}</button>
                </div>
                <div class="center",><img class="image", src="savisaacmigkairl.png",></img></div>
            },
            State::Trivia(trivia) => trivia.html(self),
            State::ChoosePerson => {
                use Person::*;
                html! {
                    <div class="question font center",>
                        {"Who are you?"}
                    </div>
                    {
                        for vec![Carl, Isaac, Kai, Miguel, Savannah, Guest].into_iter().map(|person| {
                            html! {
                                <div class="center",>
                                    <button class="button", type="button", onclick=|_| Msg::SetState(State::Unique(person)),>{person}</button>
                                </div>
                            }
                        })
                    }
                    <div class="center",>
                        <button class="button home", type="button", onclick=|_| Msg::SetState(State::Home),>
                            {"Home"}
                        </button>
                    </div>
                }
            }
            State::Unique(person) => {
                use Person::*;
                html! {
                    <div>{
                        match person {
                            Carl => html! {
                                <div class="question font",>
                                    {"Escuzi! Bopity boopy!"}
                                    <br></br>
                                    {"Take 3 drinks while doing an Italian gesture with your free hand."}
                                </div>
                                <div class="center",><img class="image", src="Flag_of_Italy.svg",></img></div>
                            },
                            Isaac => html! {
                                <div class="question font",>{
                                    "Do some pushups, then take a number of drinks \
                                     equal to forty minus how many pushups you did. \
                                     If you do more than forty, you may give out drinks."
                                }</div>
                            },
                            Kai => html! {
                                <div class="question font",>{
                                    "Have a conversation with Miguel in Spanish. \
                                     Miguel may decide how many drinks you take based \
                                     on how good your pronunciation, grammar, and \
                                     comprehension are."
                                }</div>
                            },
                            Miguel => {
                                let entry = self.entry.clone();
                                let submission = Arc::new(Box::new(move || Msg::SetState({
                                    if entry.trim().parse::<f32>() == Ok(1.0) {
                                        State::GiveDrinks(5)
                                    } else {
                                        State::Drink {
                                            correct: false,
                                            count: 3
                                        }
                                    }
                                })));
                                let submission2 = submission.clone();
                                html! {
                                    <div class="question font",>
                                        { "In the circuit shown below R2 = 2 kΩ." }
                                        { "Assume that the op-amp is ideal." }
                                        { "Determine the value of R1 so that the closed-loop gain, G = vO / vS = 3." }
                                    </div>
                                    <div class="center",><img class="image", src="op_amp.png",></img></div>
                                    <form onsubmit=|_| submission(),>
                                        <div class="center",>
                                            <input class="entry center", type="text", oninput=|s| Msg::Entry(s.value),></input>{"kΩ"}
                                        </div>
                                    </form>
                                    <button class="button center", type="button", onclick=|_| submission2(),>{"Submit"}</button>
                                }
                            },
                            Savannah => html! {
                                <div class="question font",>{
                                    "Come up with familial relations that relate all of the \
                                     other players, i.e. Miguel is Carl's dad. For the rest \
                                     of the game, other players must speak to eachother as if \
                                     they are actually related in the way you define. Anyone \
                                     who does not adhear must drink."
                                }</div>
                            },
                            Guest => html! {
                                <div class="question font",>{
                                    "The five founders of Savisaac Migkairl stand and look \
                                     down on you while you take 5 drinks."
                                }</div>
                            }
                        }
                    }</div>
                    <div class="center",>
                        <button class="button home", type="button", onclick=|_| Msg::SetState(State::Home),>{"Home"}</button>
                    </div>
                }
            }
            State::Drink { correct, count } => {
                html! {
                    <div class="question center font",>
                        {if *correct {
                            format!("Correct! Take only {} drink{}!", count, if *count == 1 { "" } else{ "s" })
                        } else {
                            format!("Wrong! Take {} drink{}!", count, if *count == 1 { "" } else{ "s" })
                        }}
                    </div>
                    <div class="center",>
                        <button class="button home", type="button", onclick=|_| Msg::SetState(State::Home),>{"Home"}</button>
                    </div>
                }
            }
            State::GiveDrinks(count) => {
                html! {
                    <div class="question center font",>
                        {format!("Correct! Give out {} drink{}!", count, if *count == 1 { "" } else { "s" })}
                    </div>
                    <div class="center",>
                        <button class="button home", type="button", onclick=|_| Msg::SetState(State::Home),>{"Home"}</button>
                    </div>
                }
            }
        }
    }
}

fn main() {
    yew::initialize();
    App::<Model>::new().mount_to_body();
    yew::run_loop();
}
