use anyhow::Result;
use ratatui::{prelude::*, widgets::*};
use ratatui_inputs::ResultKind;
use s_text_input_f_parser::CorrectBlocks;
use std::io::stdout;

use ratatui::{prelude::CrosstermBackend, Terminal};

#[derive(Clone, Copy, PartialEq, Debug)]
enum Submenu {
    CompleteTask,
    CreateTask,
}

fn create_paragraph(terminal: &mut Terminal<impl Backend>) -> Result<CorrectBlocks> {
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
    })?
    .expect("user should provide input"))
}

fn main() -> Result<()> {
    let mut terminal = Terminal::new(CrosstermBackend::new(stdout())).unwrap();
    let alt = alternate_screen_wrapper::AlternateScreen::enter().unwrap();

    let mut paragraphs = Vec::new();

    loop {
        let submenu = {
            let request = vec![s_text_input_f::Block::OneOf(vec![
                "complete task".into(),
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
            Submenu::CompleteTask => todo!(),
            Submenu::CreateTask => {
                let correct_paragraph = create_paragraph(&mut terminal)?;
                paragraphs.push(correct_paragraph);
            }
        }
    }

    drop(alt);
    dbg!(paragraphs);
    Ok(())
}
