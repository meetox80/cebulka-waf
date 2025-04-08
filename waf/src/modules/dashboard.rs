#![allow(non_snake_case)]

use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};
use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use lazy_static::lazy_static;
use sysinfo::{System, Cpu, CpuRefreshKind, RefreshKind, ProcessRefreshKind, ProcessesToUpdate};
use std::thread;
use std::io::{stdout, Write};
use crossterm::{
    execute,
    terminal::{Clear, ClearType, EnterAlternateScreen, LeaveAlternateScreen},
    cursor::{Hide, Show, MoveTo},
    style::{Color, SetForegroundColor, ResetColor, Print, Stylize, Attribute, SetAttribute},
    event::{poll, read, Event, KeyCode},
};
use std::ops::Add;

use crate::module::{ModuleInfo, register_module};
use crate::modules::cookie_manager::get_active_user_count;

lazy_static! {
    static ref DASHBOARD_DATA: Arc<Mutex<DashboardData>> = Arc::new(Mutex::new(DashboardData::new()));
    static ref SYSTEM_INFO: Arc<Mutex<System>> = Arc::new(Mutex::new(System::new_all()));
}

pub static REQUEST_RATE: AtomicU64 = AtomicU64::new(0);
pub static RESPONSE_RATE: AtomicU64 = AtomicU64::new(0);

#[derive(Clone)]
struct ModulePerformance {
    CpuUsage: f32,
    LastUpdate: Instant,
}

struct DashboardColors {
    Primary: Color,
    Secondary: Color,
    Accent: Color,
    Success: Color,
    Warning: Color,
    Danger: Color,
    Info: Color,
    Border: Color,
    Text: Color,
    Muted: Color,
}

struct DashboardData {
    Uptime: Instant,
    ModulePerformance: HashMap<String, ModulePerformance>,
    RequestsPerSecond: f64,
    ResponsesPerSecond: f64,
    LastCountUpdate: Instant,
    LastRequestCount: u64,
    LastResponseCount: u64,
    SelfPid: u32,
    ColorScheme: DashboardColors,
}

impl DashboardData {
    fn new() -> Self {
        Self {
            Uptime: Instant::now(),
            ModulePerformance: HashMap::new(),
            RequestsPerSecond: 0.0,
            ResponsesPerSecond: 0.0,
            LastCountUpdate: Instant::now(),
            LastRequestCount: 0,
            LastResponseCount: 0,
            SelfPid: std::process::id(),
            ColorScheme: DashboardColors {
                Primary: Color::Rgb { r: 73, g: 156, b: 228 },
                Secondary: Color::Rgb { r: 145, g: 205, b: 247 },
                Accent: Color::Rgb { r: 235, g: 203, b: 139 },
                Success: Color::Rgb { r: 163, g: 190, b: 140 },
                Warning: Color::Rgb { r: 235, g: 203, b: 139 },
                Danger: Color::Rgb { r: 191, g: 97, b: 106 },
                Info: Color::Rgb { r: 129, g: 161, b: 193 },
                Border: Color::Rgb { r: 180, g: 190, b: 200 },
                Text: Color::Rgb { r: 220, g: 220, b: 220 },
                Muted: Color::Rgb { r: 150, g: 150, b: 160 },
            },
        }
    }

    fn update_request_rate(&mut self, CurrentRequests: u64, CurrentResponses: u64) {
        let _Elapsed = self.LastCountUpdate.elapsed().as_secs_f64();
        if _Elapsed >= 1.0 {
            self.RequestsPerSecond = (CurrentRequests - self.LastRequestCount) as f64 / _Elapsed;
            self.ResponsesPerSecond = (CurrentResponses - self.LastResponseCount) as f64 / _Elapsed;
            
            self.LastRequestCount = CurrentRequests;
            self.LastResponseCount = CurrentResponses;
            self.LastCountUpdate = Instant::now();
        }
    }
}

pub fn register() {
    let _ModuleInfo = ModuleInfo {
        Name: String::from("Dashboard"),
        Version: String::from("1.0"),
        ProcessContent: |_| {},
    };
    
    register_module(_ModuleInfo);
    
    thread::spawn(|| {
        start_dashboard();
    });
}

fn format_duration(Duration: Duration) -> String {
    let _Seconds = Duration.as_secs();
    let _Days = _Seconds / 86400;
    let _Hours = (_Seconds % 86400) / 3600;
    let _Minutes = (_Seconds % 3600) / 60;
    let _Seconds = _Seconds % 60;
    
    format!("{}d {}h {}m {}s", _Days, _Hours, _Minutes, _Seconds)
}

fn draw_border(X: u16, Y: u16, Width: u16, Height: u16, Title: &str, BorderColor: Color, TitleColor: Color) {
    let _XEnd = X + Width - 1;
    let _YEnd = Y + Height - 1;
    
    execute!(stdout(), SetForegroundColor(BorderColor)).unwrap();
    execute!(stdout(), MoveTo(X, Y), Print("╭")).unwrap();
    
    let _TitleStart = X + 2;
    if !Title.is_empty() {
        execute!(stdout(), MoveTo(_TitleStart, Y), Print("┤ ")).unwrap();
        execute!(
            stdout(),
            SetForegroundColor(TitleColor),
            SetAttribute(Attribute::Bold),
            Print(Title),
            SetForegroundColor(BorderColor),
            Print(" ├")
        ).unwrap();
    }
    
    let _StartPos = if Title.is_empty() { X + 1 } else { X + Title.len() as u16 + 6 };
    for I in _StartPos..=_XEnd - 1 {
        execute!(stdout(), MoveTo(I, Y), Print("─")).unwrap();
    }
    
    execute!(stdout(), MoveTo(_XEnd, Y), Print("╮")).unwrap();
    
    for I in Y + 1..=_YEnd - 1 {
        execute!(stdout(), MoveTo(X, I), Print("│")).unwrap();
        execute!(stdout(), MoveTo(_XEnd, I), Print("│")).unwrap();
    }
    
    execute!(stdout(), MoveTo(X, _YEnd), Print("╰")).unwrap();
    
    for I in X + 1..=_XEnd - 1 {
        execute!(stdout(), MoveTo(I, _YEnd), Print("─")).unwrap();
    }
    
    execute!(stdout(), MoveTo(_XEnd, _YEnd), Print("╯")).unwrap();
    execute!(stdout(), ResetColor).unwrap();
}

fn draw_horizontal_gauge(X: u16, Y: u16, Width: u16, Value: f64, LowColor: Color, MedColor: Color, HighColor: Color, BackgroundChar: char, ForegroundChar: char) {
    let _FilledWidth = ((Width as f64 * Value.min(100.0)) / 100.0).round() as u16;
    let _Color = if Value > 80.0 { HighColor } else if Value > 50.0 { MedColor } else { LowColor };
    
    execute!(stdout(), MoveTo(X, Y), Print("[")).unwrap();
    
    for I in 0..Width {
        if I < _FilledWidth {
            execute!(stdout(), SetForegroundColor(_Color), MoveTo(X + 1 + I, Y), Print(ForegroundChar)).unwrap();
        } else {
            execute!(stdout(), SetForegroundColor(Color::DarkGrey), MoveTo(X + 1 + I, Y), Print(BackgroundChar)).unwrap();
        }
    }
    
    execute!(stdout(), ResetColor).unwrap();
    execute!(stdout(), MoveTo(X + Width + 1, Y), Print("]")).unwrap();
}

fn draw_sparkline(X: u16, Y: u16, Width: u16, Values: &[f64], MaxValue: f64, Color: Color) {
    let _Chars = ['▁', '▂', '▃', '▄', '▅', '▆', '▇', '█'];
    let _VLen = Values.len();
    let _DisplayPoints = Width.min(_VLen as u16);
    
    execute!(stdout(), SetForegroundColor(Color)).unwrap();
    
    for I in 0.._DisplayPoints as usize {
        let _Idx = if _VLen <= _DisplayPoints as usize {
            I
        } else {
            (_VLen - _DisplayPoints as usize) + I
        };
        
        if _Idx < Values.len() {
            let _NormalizedValue = if MaxValue > 0.0 { (Values[_Idx] / MaxValue).min(1.0) } else { 0.0 };
            let _CharIdx = (_NormalizedValue * 7.0).round() as usize;
            execute!(stdout(), MoveTo(X + I as u16, Y), Print(_Chars[_CharIdx])).unwrap();
        }
    }
    
    execute!(stdout(), ResetColor).unwrap();
}

fn draw_stats_label(X: u16, Y: u16, Label: &str, Value: &str, LabelColor: Color, ValueColor: Color) {
    execute!(stdout(), MoveTo(X, Y), SetForegroundColor(LabelColor), Print(Label)).unwrap();
    execute!(stdout(), SetForegroundColor(ValueColor), Print(Value), ResetColor).unwrap();
}

fn get_current_timestamp() -> String {
    let _Now = SystemTime::now();
    
    if let Ok(_Duration) = _Now.duration_since(UNIX_EPOCH) {
        let _Secs = _Duration.as_secs();
        
        let _SecOfDay = _Secs % 86400;
        let _Hours = _SecOfDay / 3600;
        let _Minutes = (_SecOfDay % 3600) / 60;
        let _Seconds = _SecOfDay % 60;
        
        let _Days = _Secs / 86400;
        let _Years = 1970 + _Days / 365;
        let _YearDays = _Days % 365;
        
        let _MonthDays = [31, 28, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31];
        let mut _Month = 0;
        let mut _Day = _YearDays + 1;
        
        while _Month < 12 && _Day > _MonthDays[_Month] {
            _Day -= _MonthDays[_Month];
            _Month += 1;
        }
        
        format!("{:04}-{:02}-{:02} {:02}:{:02}:{:02}", 
                _Years, _Month + 1, _Day, _Hours, _Minutes, _Seconds)
    } else {
        String::from("----/--/-- --:--:--")
    }
}

fn start_dashboard() {
    execute!(stdout(), EnterAlternateScreen, Hide).unwrap();
    
    let mut _System = SYSTEM_INFO.lock().unwrap();
    _System.refresh_all();
    
    let mut _Exit = false;
    let mut _TerminalWidth = 100u16;
    let mut _TerminalHeight = 40u16;
    
    if let Ok((Cols, Rows)) = crossterm::terminal::size() {
        _TerminalWidth = Cols;
        _TerminalHeight = Rows;
    }
    
    let _MainStartX = 2;
    let _MainWidth = _TerminalWidth - 4;
    
    let mut _RequestHistory: Vec<f64> = vec![0.0; 60];
    let mut _ResponseHistory: Vec<f64> = vec![0.0; 60];
    let mut _CpuHistory: Vec<f64> = vec![0.0; 60];
    let mut _MemHistory: Vec<f64> = vec![0.0; 60];
    
    let mut _DashboardData = DASHBOARD_DATA.lock().unwrap();
    let _ColorScheme = &_DashboardData.ColorScheme;
    std::mem::drop(_DashboardData);
    
    while !_Exit {
        if poll(Duration::from_millis(100)).unwrap() {
            if let Event::Key(KeyEvent) = read().unwrap() {
                if KeyEvent.code == KeyCode::Char('q') {
                    _Exit = true;
                }
            }
        }
        
        if let Ok((Cols, Rows)) = crossterm::terminal::size() {
            _TerminalWidth = Cols;
            _TerminalHeight = Rows;
        }
        
        _System.refresh_cpu_specifics(CpuRefreshKind::everything());
        _System.refresh_processes(ProcessesToUpdate::All, true);
        
        let _CurrentRequests = REQUEST_RATE.load(Ordering::Relaxed);
        let _CurrentResponses = RESPONSE_RATE.load(Ordering::Relaxed);
        
        let mut _DashboardData = DASHBOARD_DATA.lock().unwrap();
        _DashboardData.update_request_rate(_CurrentRequests, _CurrentResponses);
        
        _RequestHistory.push(_DashboardData.RequestsPerSecond);
        _RequestHistory.remove(0);
        _ResponseHistory.push(_DashboardData.ResponsesPerSecond);
        _ResponseHistory.remove(0);
        
        execute!(stdout(), Clear(ClearType::All)).unwrap();
        
        let _ColorScheme = &_DashboardData.ColorScheme;
        let _HeaderY = 1;
        let _MainWidth = _TerminalWidth - 4;
        
        draw_border(
            _MainStartX, 
            _HeaderY, 
            _MainWidth, 
            5, 
            "cebulka-waf", 
            _ColorScheme.Border, 
            _ColorScheme.Primary
        );
        
        let _Uptime = _DashboardData.Uptime.elapsed();
        let _UptimeStr = format_duration(_Uptime);
        
        execute!(
            stdout(),
            MoveTo(_MainStartX + 3, _HeaderY + 2),
            SetForegroundColor(_ColorScheme.Text),
            Print("System Uptime:"),
            SetForegroundColor(_ColorScheme.Success),
            Print(format!(" {}", _UptimeStr)),
            ResetColor
        ).unwrap();
        
        let _Timestamp = get_current_timestamp();
        execute!(
            stdout(),
            MoveTo(_MainStartX + _MainWidth - _Timestamp.len() as u16 - 3, _HeaderY + 2),
            SetForegroundColor(_ColorScheme.Success),
            Print(_Timestamp),
            ResetColor
        ).unwrap();
        
        let _SelfPid = _DashboardData.SelfPid;
        let _Process = _System.processes().get(&sysinfo::Pid::from(_SelfPid as usize));
        
        let (_ProcessCpuUsage, _ProcessMemoryUsage, _ProcessMemoryKb) = if let Some(Proc) = _Process {
            (
                Proc.cpu_usage(),
                (Proc.memory() as f64 / _System.total_memory() as f64) * 100.0,
                Proc.memory() / 1024
            )
        } else {
            (0.0, 0.0, 0)
        };
        
        _CpuHistory.push(_ProcessCpuUsage as f64);
        _CpuHistory.remove(0);
        _MemHistory.push(_ProcessMemoryUsage);
        _MemHistory.remove(0);
        
        let _MaxCpuValue = _CpuHistory.iter().fold(1.0, |A, &B| f64::max(A, B));
        let _MaxMemValue = _MemHistory.iter().fold(1.0, |A, &B| f64::max(A, B));
        let _MaxRequestValue = _RequestHistory.iter().fold(1.0, |A, &B| f64::max(A, B));
        let _MaxResponseValue = _ResponseHistory.iter().fold(1.0, |A, &B| f64::max(A, B));
        
        let _SysInfoY = _HeaderY + 7;
        let _SysInfoWidth = _MainWidth / 2 - 2;
        let _SysInfoHeight = 12;
        
        draw_border(
            _MainStartX, 
            _SysInfoY, 
            _SysInfoWidth, 
            _SysInfoHeight, 
            ".performance", 
            _ColorScheme.Border, 
            _ColorScheme.Accent
        );
        
        execute!(
            stdout(),
            MoveTo(_MainStartX + 3, _SysInfoY + 2),
            SetForegroundColor(_ColorScheme.Text),
            Print("CPU Usage: ")
        ).unwrap();
        
        draw_horizontal_gauge(
            _MainStartX + 14, 
            _SysInfoY + 2, 
            25, 
            _ProcessCpuUsage as f64,
            _ColorScheme.Success,
            _ColorScheme.Warning, 
            _ColorScheme.Danger,
            '░',
            '█'
        );
        
        execute!(
            stdout(),
            MoveTo(_MainStartX + 3, _SysInfoY + 3),
            SetForegroundColor(_ColorScheme.Text),
            Print("Memory:   ")
        ).unwrap();
        
        draw_horizontal_gauge(
            _MainStartX + 14, 
            _SysInfoY + 3,
            25, 
            _ProcessMemoryUsage,
            _ColorScheme.Success,
            _ColorScheme.Warning, 
            _ColorScheme.Danger,
            '░',
            '█'
        );
        
        execute!(
            stdout(),
            MoveTo(_MainStartX + 3, _SysInfoY + 4),
            SetForegroundColor(_ColorScheme.Text),
            Print("RAM Usage:"),
            SetForegroundColor(_ColorScheme.Info),
            Print(format!(" {} KB", _ProcessMemoryKb)),
            ResetColor
        ).unwrap();
        
        execute!(
            stdout(),
            MoveTo(_MainStartX + 3, _SysInfoY + 5),
            SetForegroundColor(_ColorScheme.Text),
            Print("CPU History: ")
        ).unwrap();
        
        draw_sparkline(
            _MainStartX + 16, 
            _SysInfoY + 5,
            40, 
            &_CpuHistory, 
            _MaxCpuValue, 
            _ColorScheme.Primary
        );
        
        let _RequestStatsX = _MainStartX + _SysInfoWidth + 4;
        let _RequestStatsY = _SysInfoY;
        let _RequestStatsWidth = _SysInfoWidth;
        let _RequestStatsHeight = _SysInfoHeight;
        
        draw_border(
            _RequestStatsX, 
            _RequestStatsY, 
            _RequestStatsWidth, 
            _RequestStatsHeight, 
            ".traffic", 
            _ColorScheme.Border, 
            _ColorScheme.Accent
        );
        
        draw_stats_label(
            _RequestStatsX + 3, 
            _RequestStatsY + 2, 
            "Requests/sec: ", 
            &format!("{:8.2}", _DashboardData.RequestsPerSecond),
            _ColorScheme.Text,
            _ColorScheme.Success
        );
        
        draw_stats_label(
            _RequestStatsX + 30, 
            _RequestStatsY + 2, 
            "Total Requests: ", 
            &format!("{}", _CurrentRequests),
            _ColorScheme.Text,
            _ColorScheme.Primary
        );
        
        draw_stats_label(
            _RequestStatsX + 3, 
            _RequestStatsY + 3,
            "Responses/sec: ", 
            &format!("{:8.2}", _DashboardData.ResponsesPerSecond),
            _ColorScheme.Text,
            _ColorScheme.Success
        );
        
        draw_stats_label(
            _RequestStatsX + 30, 
            _RequestStatsY + 3,
            "Total Responses: ", 
            &format!("{}", _CurrentResponses),
            _ColorScheme.Text,
            _ColorScheme.Primary
        );
        
        execute!(
            stdout(),
            MoveTo(_RequestStatsX + 3, _RequestStatsY + 4),
            SetForegroundColor(_ColorScheme.Text),
            Print("Request History: ")
        ).unwrap();
        
        draw_sparkline(
            _RequestStatsX + 20, 
            _RequestStatsY + 4,
            40, 
            &_RequestHistory, 
            _MaxRequestValue, 
            _ColorScheme.Info
        );
        
        execute!(
            stdout(),
            MoveTo(_RequestStatsX + 3, _RequestStatsY + 5),
            SetForegroundColor(_ColorScheme.Text),
            Print("Response History:")
        ).unwrap();
        
        draw_sparkline(
            _RequestStatsX + 20, 
            _RequestStatsY + 5,
            40, 
            &_ResponseHistory, 
            _MaxResponseValue, 
            _ColorScheme.Accent
        );
        
        let _UserStatsY = _SysInfoY + _SysInfoHeight + 1;
        let _UserStatsWidth = _MainWidth;
        let _UserStatsHeight = 6;
        
        draw_border(
            _MainStartX, 
            _UserStatsY, 
            _UserStatsWidth, 
            _UserStatsHeight, 
            ".users",
            _ColorScheme.Border, 
            _ColorScheme.Accent
        );
        
        let _ActiveUsers = get_active_user_count();
        let _Col1Width = _UserStatsWidth / 3;
        
        draw_stats_label(
            _MainStartX + 3, 
            _UserStatsY + 2, 
            "Authenticated Users: ", 
            &format!("{}", _ActiveUsers),
            _ColorScheme.Text,
            _ColorScheme.Primary
        );
        
        let _AuthRatio = if _CurrentRequests > 0 {
            (_ActiveUsers as f64 / _CurrentRequests as f64) * 100.0
        } else {
            0.0
        };
        
        draw_stats_label(
            _MainStartX + _Col1Width + 5, 
            _UserStatsY + 2, 
            "Auth/Request Ratio: ", 
            &format!("{:.2}%", _AuthRatio),
            _ColorScheme.Text,
            _ColorScheme.Info
        );
        
        let _SuccessRate = if _CurrentRequests > 0 {
            (_CurrentResponses as f64 / _CurrentRequests as f64) * 100.0
        } else {
            0.0
        };
        
        draw_stats_label(
            _MainStartX + 2 * _Col1Width + 5, 
            _UserStatsY + 2, 
            "Success Rate: ", 
            &format!("{:.2}%", _SuccessRate),
            _ColorScheme.Text,
            _ColorScheme.Success
        );
        
        draw_horizontal_gauge(
            _MainStartX + 3, 
            _UserStatsY + 4, 
            _UserStatsWidth - 6, 
            _SuccessRate,
            _ColorScheme.Danger,
            _ColorScheme.Warning, 
            _ColorScheme.Success,
            '░',
            '█'
        );
        
        let _ModuleY = _UserStatsY + _UserStatsHeight + 1;
        
        let _RemainingHeight = if _TerminalHeight > _ModuleY + 4 {
            _TerminalHeight - _ModuleY - 4
        } else {
            3
        };
        
        let _ModuleHeight = _RemainingHeight;
        
        draw_border(
            _MainStartX, 
            _ModuleY, 
            _MainWidth, 
            _ModuleHeight, 
            "MODULE PERFORMANCE", 
            _ColorScheme.Border, 
            _ColorScheme.Primary
        );
        
        let _MaxModulesPerCol = if _ModuleHeight > 3 { (_ModuleHeight - 3) as usize } else { 1 };
        let _ModuleColWidth = (_MainWidth - 6) / 3;
        
        let mut _Row = 0;
        let mut _Col = 0;
        
        let _ModulePerformance = _DashboardData.ModulePerformance.clone();
        
        for (_ModuleName, _Performance) in _ModulePerformance.iter().take(_MaxModulesPerCol * 3) {
            let _XPos = _MainStartX + 3 + ((_Col * _ModuleColWidth as usize) as u16);
            let _YPos = _ModuleY + 2 + (_Row % _MaxModulesPerCol) as u16;
            
            execute!(
                stdout(),
                MoveTo(_XPos, _YPos),
                SetForegroundColor(_ColorScheme.Text),
                Print(format!("{}: ", _ModuleName))
            ).unwrap();
            
            draw_horizontal_gauge(
                _XPos + 15, 
                _YPos, 
                _ModuleColWidth - 25, 
                _Performance.CpuUsage as f64,
                _ColorScheme.Success,
                _ColorScheme.Warning, 
                _ColorScheme.Danger,
                '░',
                '█'
            );
            
            _Row += 1;
            if _Row % _MaxModulesPerCol == 0 {
                _Col += 1;
                if _Col >= 3 {
                    break;
                }
            }
        }
        
        draw_border(
            _MainStartX, 
            _TerminalHeight - 3, 
            _MainWidth, 
            3, 
            "", 
            _ColorScheme.Border, 
            _ColorScheme.Primary
        );
        
        execute!(
            stdout(),
            MoveTo(_MainStartX + 3, _TerminalHeight - 2),
            SetForegroundColor(_ColorScheme.Muted),
            Print("Press 'q' to exit dashboard | Refresh Rate: 500ms | cebulka-waf security dashboard"),
            ResetColor
        ).unwrap();
        
        stdout().flush().unwrap();
        thread::sleep(Duration::from_millis(500));
    }
    
    execute!(stdout(), LeaveAlternateScreen, Show).unwrap();
}

pub fn update_module_performance(ModuleName: &str, CpuUsage: f32) {
    let mut _Data = DASHBOARD_DATA.lock().unwrap();
    _Data.ModulePerformance.insert(ModuleName.to_string(), ModulePerformance {
        CpuUsage,
        LastUpdate: Instant::now(),
    });
}

pub fn increment_request_counter() {
    REQUEST_RATE.fetch_add(1, Ordering::Relaxed);
}

pub fn increment_response_counter() {
    RESPONSE_RATE.fetch_add(1, Ordering::Relaxed);
} 