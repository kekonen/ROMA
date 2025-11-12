use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::{Backend, CrosstermBackend},
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph},
    Frame, Terminal,
};
use roma_core::{TaskNode, TaskStatus};
use roma_engine::TaskDag;
use std::io;

pub struct TuiApp {
    dag: TaskDag,
    selected_task: Option<String>,
}

impl TuiApp {
    pub fn new(dag: TaskDag) -> Self {
        Self {
            dag,
            selected_task: None,
        }
    }

    pub async fn run(mut self) -> anyhow::Result<()> {
        enable_raw_mode()?;
        let mut stdout = io::stdout();
        execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
        let backend = CrosstermBackend::new(stdout);
        let mut terminal = Terminal::new(backend)?;

        let res = self.run_app(&mut terminal).await;

        disable_raw_mode()?;
        execute!(
            terminal.backend_mut(),
            LeaveAlternateScreen,
            DisableMouseCapture
        )?;
        terminal.show_cursor()?;

        if let Err(err) = res {
            println!("Error: {:?}", err);
        }

        Ok(())
    }

    async fn run_app<B: Backend>(&mut self, terminal: &mut Terminal<B>) -> io::Result<()> {
        loop {
            terminal.draw(|f| self.ui(f))?;

            if event::poll(std::time::Duration::from_millis(100))? {
                if let Event::Key(key) = event::read()? {
                    match key.code {
                        KeyCode::Char('q') => return Ok(()),
                        KeyCode::Char('j') | KeyCode::Down => self.select_next(),
                        KeyCode::Char('k') | KeyCode::Up => self.select_previous(),
                        _ => {}
                    }
                }
            }
        }
    }

    fn ui(&self, f: &mut Frame) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Percentage(70), Constraint::Percentage(30)].as_ref())
            .split(f.area());

        let tasks = self.dag.get_all_tasks();

        let items: Vec<ListItem> = tasks
            .iter()
            .map(|task| {
                let status_color = match task.status {
                    TaskStatus::Completed => Color::Green,
                    TaskStatus::Failed => Color::Red,
                    TaskStatus::Executing => Color::Yellow,
                    TaskStatus::Ready => Color::Cyan,
                    TaskStatus::Pending => Color::Gray,
                };

                let content = Line::from(vec![
                    Span::styled(
                        format!("{:?} ", task.status),
                        Style::default()
                            .fg(status_color)
                            .add_modifier(Modifier::BOLD),
                    ),
                    Span::raw(&task.goal),
                ]);

                ListItem::new(content)
            })
            .collect();

        let list = List::new(items)
            .block(Block::default().borders(Borders::ALL).title("Task DAG"))
            .highlight_style(Style::default().bg(Color::DarkGray));

        f.render_widget(list, chunks[0]);

        let detail_text = if let Some(task_id) = &self.selected_task {
            if let Ok(task) = self.dag.get_task(task_id) {
                format!(
                    "Task ID: {}\nGoal: {}\nStatus: {:?}\nDepth: {}\nResult: {}",
                    task.task_id,
                    task.goal,
                    task.status,
                    task.depth,
                    task.result.as_deref().unwrap_or("<none>")
                )
            } else {
                "No task selected".to_string()
            }
        } else {
            "No task selected\n\nPress 'q' to quit, 'j/k' or arrow keys to navigate".to_string()
        };

        let detail = Paragraph::new(detail_text)
            .block(Block::default().borders(Borders::ALL).title("Task Details"));

        f.render_widget(detail, chunks[1]);
    }

    fn select_next(&mut self) {
        let tasks = self.dag.get_all_tasks();
        if tasks.is_empty() {
            return;
        }

        if let Some(current) = &self.selected_task {
            if let Some(pos) = tasks.iter().position(|t| &t.task_id == current) {
                if pos + 1 < tasks.len() {
                    self.selected_task = Some(tasks[pos + 1].task_id.clone());
                }
            }
        } else {
            self.selected_task = Some(tasks[0].task_id.clone());
        }
    }

    fn select_previous(&mut self) {
        let tasks = self.dag.get_all_tasks();
        if tasks.is_empty() {
            return;
        }

        if let Some(current) = &self.selected_task {
            if let Some(pos) = tasks.iter().position(|t| &t.task_id == current) {
                if pos > 0 {
                    self.selected_task = Some(tasks[pos - 1].task_id.clone());
                }
            }
        } else {
            self.selected_task = Some(tasks[0].task_id.clone());
        }
    }
}
