use evalexpr::*;
// use libm::*;
use teloxide::{
    dispatching::dialogue::{serializer::Bincode, ErasedStorage, SqliteStorage, Storage},
    prelude::*,
    types::ParseMode,
};

type MyDialogue = Dialogue<State, ErasedStorage<State>>;
type MyStorage = std::sync::Arc<ErasedStorage<State>>;
type HandlerResult = Result<(), Box<dyn std::error::Error + Send + Sync>>;

#[derive(Clone, Default, serde::Serialize, serde::Deserialize, Debug)]
pub enum State {
    #[default]
    Start,
    ReceiveExpression(Option<evalexpr::HashMapContext>),
}

#[tokio::main]
async fn main() {
    let bot = Bot::from_env();
    let storage: MyStorage = SqliteStorage::open("db.sqlite", Bincode)
        .await
        .unwrap()
        .erase();
    let handler = Update::filter_message()
        .enter_dialogue::<Message, ErasedStorage<State>, State>()
        .branch(dptree::case![State::Start].endpoint(start))
        .branch(dptree::case![State::ReceiveExpression(ctx)].endpoint(receive_expression));
    Dispatcher::builder(bot, handler)
        .dependencies(dptree::deps![storage])
        .build()
        .dispatch()
        .await;
}
async fn start(bot: Bot, dialogue: MyDialogue, msg: Message) -> HandlerResult {
    bot.send_message(
        msg.chat.id,
        r#"Hey there\! Enter an expression or a command /help for help\."#,
    )
    .parse_mode(ParseMode::MarkdownV2)
    .await?;
    dialogue.update(State::ReceiveExpression(None)).await?;
    Ok(())
}

async fn receive_expression(bot: Bot, dialogue: MyDialogue, msg: Message) -> HandlerResult {
    match dialogue.get().await.unwrap().unwrap() {
        State::ReceiveExpression(ctx) => {
            let mut ctx = match ctx {
                Some(ctx) => ctx,
                None => HashMapContext::new(),
            };
            let mut clear: bool = false;
            let mut exit: bool = false;
            let msgtext = match msg.text() {
                Some(text) => match text {
                    "/help" => {
                        println!("{text}: Help requested.");
                        format!(
                            r#"Enter an expression or a command\.
Example 1: `math::sin(0.5) + math::cos(0.5)`
Example 2: /functions
Example 3: `a = 1; b = 2; a + b`
Example 4: `test_str = "Hello, World!"; str::to_uppercase(test_str)`
Available commands: /help, /clear, /start, /exit, /context, /functions, /operators"#
                        )
                    }
                    "/clear" => {
                        println!("{text}: Context cleared.");
                        clear = true;
                        format!(r#"Context cleared\."#)
                    }
                    "/start" => {
                        println!("{text}: Start requested.");
                        format!(r#"Enter an expression or a command\. /help for help\!"#)
                    }
                    "/exit" => {
                        println!("{text}: Exit requested.");
                        exit = true;
                        format!(r#"Goodbye\!"#)
                    }
                    "/context" => {
                        println!("{text}: Context requested: {ctx:?}");
                        format!("`{ctx:?}`")
                            // .replace(".", r#"\."#).replace("[", r#"\["#).replace("]", r#"\]"#)
                            // .replace("{", r#"\{"#).replace("}", r#"\}"#)
                            // .replace("(", r#"\("#).replace(")", r#"\)"#)
                            // .replace("_", r#"\_"#).replace("*", r#"\*"#)
                            // .replace("~", r#"\~"#).replace("`", r#"\`"#)
                            // .replace(">", r#"\>"#).replace("#", r#"\#"#)
                            // .replace("+", r#"\+"#).replace("-", r#"\-"#)
                            // .replace("|", r#"\|"#).replace("!", r#"\!"#)
                    }
                    "/functions" => {
                        println!("{text}: Functions requested.");
                        format!(
                            r#"Available functions:
min, max, len, floor, round, ceil, if, contains, contains\_any, typeof, math::is\_nan, math::is\_finite, math::is\_infinite, math::is\_normal, math::ln, math::log, math::log2, math::log10, math::exp, math::exp2, math::pow, math::cos, math::acos, math::cosh, math::acosh, math::sin, math::asin, math::sinh, math::asinh, math::tan, math::atan, math::atan2, math::tanh, math::atanh, math::sqrt, math::cbrt, math::hypot, math::abs, str::regex\_matches, str::regex\_replace, str::to\_lowercase, str::to\_uppercase, str::trim, str::from, str::substring, bitand, bitor, bitxor, bitnot, shl, shr, random

For extra info, refer to https://crates\.io/crates/evalexpr"#
                        )
                    }
                    "/operators" => {
                        println!("{text}: Operators requested.");
                        format!(
                            r#"operators, their priorities and their meanings:
```
^(120)  — exponentiation
*(100)  — product
/(100)  — division(integer if both args are integers, float otherwise)
%(100)  — modulo
+(95)   — addition or string concatenation
-(95)   — subtraction
<(80)   — less than
>(80)   — greater than
<=(80)  — less than or equal to
>=(80)  — greater than or equal to
==(80)  — equal to
!=(80)  — not equal to
&&(75)  — logical and
||(70)  — logical or
=(50)   — assignment
+=(50)  — addition assignment
-=(50)  — subtraction assignment
*=(50)  — multiplication assignment
/=(50)  — division assignment
%=(50)  — modulo assignment
^=(50)  — exponentiation assignment
&&=(50) — logical and assignment
||=(50) — logical or assignment
,(40)   — aggregation
;(0)    — statement separator
```

For extra info, refer to https://crates\.io/crates/evalexpr"#
                        )
                    }

                    _ => {
                        let (dbgmsg, expr) = match evalexpr::build_operator_tree(text) {
                            Ok(optree) => (format!(r#"{text}: {optree}"#), Some(optree)),
                            Err(_emsg) => (format!(r#"{text}: Invalid expression!"#), None),
                        };
                        match expr {
                            Some(expr) => match expr.eval_with_context_mut(&mut ctx) {
                                Ok(val) => {
                                    println!("Ok:({text}): {val}");
                                    format!(r#"`{val}`"#)
                                }
                                Err(emsg) => {
                                    println!("Error:({text}): {emsg}");
                                    format!(r#"`{emsg}`"#)
                                }
                            },
                            None => {
                                println!("Error:({text}): {dbgmsg}");
                                format!(r#"`{dbgmsg}`"#)
                            }
                        }
                    }
                },
                None => format!(r#"No text received\. /help for help\!"#),
            };
            bot.send_message(msg.chat.id, msgtext)
                .parse_mode(ParseMode::MarkdownV2)
                .await?;
            if exit {
                dialogue.exit().await?;
            } else {
                dialogue
                    .update(State::ReceiveExpression(if clear {
                        None
                    } else {
                        Some(ctx)
                    }))
                    .await?
            }
        }
        _ => unreachable!(),
    };

    Ok(())
}
