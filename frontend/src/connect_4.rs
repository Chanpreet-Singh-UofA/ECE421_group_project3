//https://github.com/yewstack/yew/tree/master/examples/game_of_life for learning how to use yew to draw cells
#[path = "../src/connect4/mod.rs"]
mod connect4;
use connect4::connect4::Connect4;
use crate::cell::Cellule;
use rand::Rng;
use yew::html::Scope;
use yew::events::Event;
use web_sys::{EventTarget, HtmlInputElement};
use yew::{classes, html, Component, Context, Html, NodeRef};
use std::{
    error::Error,
    fmt::{self, Debug, Display, Formatter},
};
use serde::{Deserialize, Serialize};
use web_sys::{Request, RequestInit, RequestMode, Response};
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::JsFuture;
use std::time::{SystemTime, UNIX_EPOCH, Duration};
use chrono;

#[derive(Serialize, Deserialize, Debug, Clone)]
#[allow(non_snake_case)]
pub struct Game {
    gameID: String,
    gameType: String,
    player1: String,
    player2: String,
    winner: String,
    playedTime: String,
}

#[derive(Debug, Clone, PartialEq)]
pub struct FetchError {
    err: JsValue,
}

impl Display for FetchError {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        Debug::fmt(&self.err, f)
    }
}
impl Error for FetchError {}

impl From<JsValue> for FetchError {
    fn from(value: JsValue) -> Self {
        Self { err: value }
    }
}

pub enum Msg {
    Reset,
    ToggleCellule(usize),
    updatePlayer1(String),
    updatePlayer2(String),
    GetOK(),
    GetFailed(String),
    
}

pub struct InputData {
    pub value: String,
}

pub struct connect_4 {
    cellules: Vec<Cellule>,
    cellules_width: usize,
    cellules_height: usize,
    player1: String,
    player2: String,
    input: NodeRef,
    input2: NodeRef,
    current_player: u8,
    board: Connect4,
    winnerString: String,
    is_game_over: bool,
}

pub async fn send_post_request(game_result:Game) -> Result<(), FetchError> {
    let mut opts = RequestInit::new();
    opts.method("POST");
    opts.mode(RequestMode::Cors);

    let game_result_json = serde_json::to_string(&game_result).unwrap();

    opts.body(Some(&JsValue::from_serde(&game_result_json).unwrap()));


    let request = Request::new_with_str_and_init("http://localhost:8000/addGame", &opts)?;

    request
        .headers()
        .set("Content-Type", "text/plain")?;
    let window = web_sys::window().unwrap();
    let resp_value = JsFuture::from(window.fetch_with_request(&request)).await?;
    

    Ok(())
}

impl connect_4 {

    fn reset(&mut self) {
        for cellule in self.cellules.iter_mut() {
            cellule.set_dead();
        }
    }

    fn row_col_as_idx(&self, row: isize, col: isize) -> usize {
        let row = wrap(row, self.cellules_height as isize);
        let col = wrap(col, self.cellules_width as isize);

        row * self.cellules_width + col
    }

    fn idx_to_row_col(&self, idx: usize) -> (isize, isize) {
        let row = idx / self.cellules_width;
        let col = idx % self.cellules_width;
        (row as isize, col as isize)
    }

    fn view_cellule(&self, idx: usize, cellule: &Cellule, link: &Scope<Self>) -> Html {
        let cellule_status = {
            if cellule.is_alive() {
                "cellule-live"
            } else if (cellule.is_dead()) {
                "cellule-dead"
            } else {
                "cellule-green"
            }
        };
        html! {
            <div key={idx} class={classes!("game-cellule", cellule_status)}
                onclick={link.callback(move |_| Msg::ToggleCellule(idx))}>
                {
                    if cellule.is_alive() {
                        "X"
                    } else if (cellule.is_dead()) {
                        ""
                    } else {
                        "O"
                    }
                }
            </div>
        }
    }
}
impl Component for connect_4 {
    type Message = Msg;
    type Properties = ();

    fn create(ctx: &Context<Self>) -> Self {

        let (cellules_width, cellules_height) = (7, 6);

        Self {
            cellules: vec![Cellule::new_dead(); cellules_width * cellules_height],
            cellules_width,
            cellules_height,
            player1: String::from(""),
            player2: String::from(""),
            input: NodeRef::default(),
            input2: NodeRef::default(),
            current_player: 1,
            board: Connect4::new(),
            winnerString: String::from(""),
            is_game_over: true,
        }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::Reset => {
                self.reset();
                log::info!("Reset");
                self.is_game_over = false;
                self.winnerString = String::from("");
                self.board = Connect4::new();
                self.current_player = 1;
                true
            }
            Msg::ToggleCellule(idx) => {
                if(!self.is_game_over){
                    let (_, col) = self.idx_to_row_col(idx);
                    if(self.board.insert(col as usize,self.current_player)){
                        let row = 6 - self.board.col_height[col as usize];
                        let index = self.row_col_as_idx(row as isize,col);
                        let cellule = self.cellules.get_mut(index).unwrap();
                        cellule.toggle(self.current_player);
                        let gameState = self.board.check_win_draw(col as usize, self.current_player);
                        if( gameState == 1){
                            if(self.current_player == 1){
                                self.winnerString = format!("{} wins!", self.player1);
                                let game_result = Game {
                                    gameID: "".to_string(),
                                    gameType: "Connect-4".to_string(),
                                    player1: self.player1.clone(),
                                    player2: self.player2.clone(),
                                    winner: self.player1.clone(),
                                    playedTime: "test time".to_string(),
                                };
                                ctx.link().send_future(async {
                                    match send_post_request(game_result).await {
                                        Ok(_) => {
                                            Msg::GetOK()
                                        },
                                        Err(err) => {
                                            Msg::GetFailed(err.to_string())
                                        }
                                    }
                                });
                            }else{
                                self.winnerString = format!("{} wins!", self.player2);
                                let game_result = Game {
                                    gameID: "".to_string(),
                                    gameType: "Connect-4".to_string(),
                                    player1: self.player1.clone(),
                                    player2: self.player2.clone(),
                                    winner: self.player2.clone(),
                                    playedTime: "test time".to_string(),
                                };
                                ctx.link().send_future(async {
                                    match send_post_request(game_result).await {
                                        Ok(_) => {
                                            Msg::GetOK()
                                        },
                                        Err(err) => {
                                            Msg::GetFailed(err.to_string())
                                        }
                                    }
                                });
                            }
                            self.is_game_over = true;
                        }
                        else if( gameState == -1){
                            self.winnerString = String::from("Draw");
                            self.is_game_over = true;
                            let game_result = Game {
                                gameID: "".to_string(),
                                gameType: "Connect-4".to_string(),
                                player1: self.player1.clone(),
                                player2: self.player2.clone(),
                                winner: "Draw".to_string(),
                                playedTime: "test time".to_string(),
                            };
                            ctx.link().send_future(async {
                                match send_post_request(game_result).await {
                                    Ok(_) => {
                                        Msg::GetOK()
                                    },
                                    Err(err) => {
                                        Msg::GetFailed(err.to_string())
                                    }
                                }
                            });
                        }
                        else{
                            if(self.current_player == 1){
                                self.current_player = 2;
                            }else{
                                self.current_player = 1;
                            }
                        }
                        return true;
                    }
                }
                false

            }
            Msg::updatePlayer1(player1) => {
                if(self.is_game_over){
                    self.player1 = player1;
                    return true;
                }
                false
            }
            Msg::updatePlayer2(player2) => {
                if(self.is_game_over){
                    self.player2 = player2;
                    return true;
                }
                false    
            },
            Msg::GetOK() => {
                false
            },
            Msg::GetFailed(err) => {
                false
            }
            
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        
        let my_input_ref = self.input.clone();
        let my_input_ref2 = self.input2.clone();
        let onchange = ctx.link().batch_callback(move |_| {
            let input = my_input_ref.cast::<HtmlInputElement>();
            input.map(|input| Msg::updatePlayer1(input.value()))
        });

        let onchange2 = ctx.link().batch_callback(move |_| {
            let input = my_input_ref2.cast::<HtmlInputElement>();

            input.map(|input| Msg::updatePlayer2(input.value()))
        });
        let cell_rows =
            self.cellules
                .chunks(self.cellules_width)
                .enumerate()
                .map(|(y, cellules)| {
                    let idx_offset = y * self.cellules_width;

                    let cells = cellules
                        .iter()
                        .enumerate()
                        .map(|(x, cell)| self.view_cellule(idx_offset + x, cell, ctx.link()));
                    html! {
                        <div key={y} class="game-row">
                            { for cells }
                        </div>
                    }
                });

        html! {
            <div>
                <section class="game-container">
                    <header class="app-header">
                        <h1 class="app-title">{ "Connect4 PvP" }</h1>
                    </header>
                    <section class="game-area">

                        <div class="game-of-life">
                            { for cell_rows }
                        </div>
                        <div class="game-buttons">
                            <input
                                ref={self.input.clone()}
                                id="textbox1"
                                type="text"
                                placeholder="Player 1's Name"
                                onchange = {onchange}
                            />
                            <input
                                ref={self.input2.clone()}
                                id="textbox2"
                                type="text"
                                placeholder="Player 2's Name"
                                //read text from textbox
                                onchange = {onchange2}
                            />
                            <button class="game-button" onclick={ctx.link().callback(|_| Msg::Reset)}>{ "Start" }</button>
                        </div>
                        <div class="readout">
                            <div class = "text2">
                                {format!("Player 1: {}   |   Player 2: {}", self.player1,self.player2)}
                            </div>
                            <div class = "text2">
                                {format!("Current Turn: {}", {if(self.current_player == 1){self.player1.clone()}else{self.player2.clone()}})}
                            </div>
                            <div class = "text2">
                                {format!("{}", self.winnerString)}
                            </div>
                        </div>
                    </section>
                </section>
                <footer class="app-footer">
                    <strong class="footer-text">
                      { "" }
                    </strong>
                </footer>
            </div>
        }
    }
}

fn wrap(coord: isize, range: isize) -> usize {
    let result = if coord < 0 {
        coord + range
    } else if coord >= range {
        coord - range
    } else {
        coord
    };
    result as usize
}
