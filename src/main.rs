use anyhow::Result;
use ratatui::{
    prelude::{CrosstermBackend, *},
    widgets::*,
    Terminal,
};
use ratatui_inputs::ResultKind;
use s_text_input_f_parser::CorrectBlocks;
use ssr_core::tasks_facade::TasksFacade;
use std::{
    io::{stdout, Write},
    process::ExitCode,
};

type Task = ssr_algorithms::super_memory_2::WriteAnswer;
type Facade<'a> = ssr_facade::Facade<'a, Task>;

#[derive(Clone, Copy, PartialEq, Debug)]
enum Submenu {
    CompleteTask,
    CreateTask,
}

fn create_task(terminal: &mut Terminal<impl Backend>) -> Result<Option<CorrectBlocks>> {
    Ok(ratatui_inputs::get_blocks(&mut |styled, support_text| {
        let layout = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Fill(1), Constraint::Fill(1)]);
        terminal
            .draw(|f| {
                let layout = layout.split(f.area());

                let input_block = ratatui::widgets::Block::bordered()
                    .border_type(ratatui::widgets::BorderType::Rounded);
                let input_area = input_block.inner(layout[0]);

                let support_block = ratatui::widgets::Block::new().padding(Padding::uniform(1));
                let support_area = support_block.inner(layout[1]);

                f.render_widget(input_block, layout[0]);
                f.render_widget(
                    ratatui::widgets::Paragraph::new(styled).wrap(Wrap { trim: true }),
                    input_area,
                );
                f.render_widget(support_block, layout[1]);
                f.render_widget(ratatui::widgets::Paragraph::new(support_text), support_area);
            })
            .map(|_| ())
    })?)
}

use clap::{Parser, Subcommand};
#[derive(Parser, Debug)]
struct Args {
    #[command(subcommand)]
    action: Option<Action>,
}

#[derive(Subcommand, Debug)]
enum Action {
    Add { content: String },
}

fn main() -> Result<ExitCode> {
    let args = Args::parse();

    let path = "storage.json";
    let file = std::fs::read_to_string(path);
    let mut storage: Facade = if let Ok(file) = &file {
        serde_json::from_str(file)?
    } else {
        Facade::new("test_name".into())
    };

    let success = if let Some(action) = args.action {
        match action {
            Action::Add { content } => {
                let a = s_text_input_f_parser::parse_blocks(&content);
                match a {
                    Ok(blocks) => {
                        if blocks.answer.iter().map(|x| x.len()).sum::<usize>() == 0 {
                            eprintln!("Task must contain interactive elements.");
                            false
                        } else {
                            let task = Task::new(blocks.blocks, blocks.answer);
                            storage.insert(task);
                            true
                        }
                    }
                    Err(errs) => {
                        for err in errs {
                            eprintln!("Parsing error: {err}.");
                        }
                        false
                    }
                }
            }
        }
    } else {
        application(&mut storage)?;
        true
    };

    save(path, storage)?;

    if success {
        Ok(ExitCode::SUCCESS)
    } else {
        Ok(ExitCode::FAILURE)
    }
}
fn application(storage: &mut Facade) -> Result<()> {
    let mut terminal = Terminal::new(CrosstermBackend::new(stdout())).unwrap();
    let alt = alternate_screen_wrapper::AlternateScreen::enter()?.bracketed_paste()?;

    loop {
        let submenu = {
            storage.find_tasks_to_recall();
            let request = vec![s_text_input_f::Block::OneOf(vec![
                format!("complete task ({})", {
                    let to_complete = storage.tasks_to_complete();
                    if to_complete > 0 {
                        to_complete.to_string()
                    } else {
                        let until = storage.until_next_repetition();
                        if let Some(until) = until {
                            format!("0; {:.2}h", until.as_secs_f64() / 3600.)
                        } else {
                            0.to_string()
                        }
                    }
                }),
                "create task".into(),
            ])];
            let (result_kind, answer) = ratatui_inputs::get_input(request, &mut |text| {
                terminal
                    .draw(|f| f.render_widget(Paragraph::new(text), f.area()))
                    .map(|_| ())
            })
            .unwrap()?;

            if result_kind == ResultKind::Canceled {
                break;
            }
            let answer: usize = answer[0][0].parse()?;
            [Submenu::CompleteTask, Submenu::CreateTask][answer]
        };
        match submenu {
            Submenu::CompleteTask => {
                complete_task(storage, &mut terminal);
            }
            Submenu::CreateTask => {
                let correct_blocks = create_task(&mut terminal)?;
                if let Some(task) = correct_blocks {
                    let task = Task::new(task.blocks, task.answer);
                    storage.insert(task);
                }
            }
        }
    }
    drop(alt);
    Ok(())
}

fn complete_task(
    storage: &mut ssr_facade::Facade<'_, ssr_algorithms::super_memory_2::WriteAnswer>,
    terminal: &mut Terminal<CrosstermBackend<std::io::Stdout>>,
) {
    let _ = storage.complete_task(&mut |blocks| {
        let (_result_kind, answer) = ratatui_inputs::get_input(blocks, &mut |text| {
            terminal
                .draw(|f| f.render_widget(text, f.area()))
                .map(|_| ())
        })
        .transpose()?
        .unwrap_or((ResultKind::Ok, vec![vec![]]));
        Ok(answer)
    });
}

// FIXME: first create file, than rename it to `path` to not corrupt data
fn save(path: &str, storage: Facade) -> Result<()> {
    writeln!(
        std::fs::File::create(path)?,
        "{}",
        serde_json::to_string_pretty(&storage)?
    )?;
    Ok(())
}
