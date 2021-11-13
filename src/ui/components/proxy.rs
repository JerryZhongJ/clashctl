use std::{collections::HashMap, hash::Hash};
use std::{fmt::Debug, marker::PhantomData};

use tui::{
    style::{Color, Modifier, Style},
    text::{Span, Spans},
    widgets::{Paragraph, Widget},
};

use crate::{
    components::{get_hash, Consts},
    model::{History, Proxies, Proxy, ProxyType},
    ui::components::{get_block, get_focused_block, get_text_style},
};

#[derive(Clone, Debug, Hash)]
pub struct ProxyGroup<'a> {
    pub name: String,
    pub proxy_type: ProxyType,
    pub members: Vec<ProxyItem>,
    pub current: Option<usize>,
    pub cursor: usize,
    pub(crate) _life: PhantomData<&'a ()>,
}

pub enum ProxyGroupFocusStatus {
    None,
    Focused,
    Expanded,
}

impl<'a> ProxyGroup<'a> {
    pub(crate) fn get_summary_widget(&self) -> impl Iterator<Item = Span> {
        self.members.iter().map(|x| {
            if x.proxy_type.is_normal() {
                match x.history {
                    Some(History { delay, .. }) => Self::get_delay_span(delay),
                    None => Consts::NO_LATENCY_SPAN,
                }
            } else {
                Consts::NOT_PROXY_SPAN
            }
        })
    }

    pub(crate) fn get_widget(
        &'a self,
        width: usize,
        status: ProxyGroupFocusStatus,
    ) -> Vec<Spans<'a>> {
        let delimiter = Span::raw(" ");
        let prefix = if matches!(status, ProxyGroupFocusStatus::Focused) {
            Consts::FOCUSED_INDICATOR_SPAN
        } else {
            Consts::UNFOCUSED_INDICATOR_SPAN
        };
        let name = Span::styled(
            &self.name,
            Style::default()
                .fg(Color::White)
                .add_modifier(Modifier::BOLD),
        );

        let proxy_type = Span::styled(self.proxy_type.to_string(), Consts::PROXY_TYPE_STYLE);

        let count = self.members.len();
        let proxy_count = Span::styled(
            if matches!(status, ProxyGroupFocusStatus::Expanded) {
                format!("{}/{}", self.cursor + 1, count)
            } else {
                count.to_string()
            },
            Style::default().fg(Color::Green),
        );

        let mut ret = Vec::with_capacity(if matches!(status, ProxyGroupFocusStatus::Expanded) {
            self.members.len() + 1
        } else {
            2
        });

        ret.push(Spans::from(vec![
            prefix.clone(),
            name,
            delimiter.clone(),
            proxy_type,
            delimiter,
            proxy_count,
        ]));

        if matches!(status, ProxyGroupFocusStatus::Expanded) {
            let skipped = self.cursor.saturating_sub(4);
            let text_style = get_text_style();
            let is_current =
                |index: usize| self.current.map(|x| x == index + skipped).unwrap_or(false);
            let is_pointed = |index: usize| self.cursor == index + skipped;

            let lines = self.members.iter().skip(skipped).enumerate().map(|(i, x)| {
                let prefix = if self.cursor == i + skipped {
                    Consts::EXPANDED_FOCUSED_INDICATOR_SPAN
                } else {
                    Consts::EXPANDED_INDICATOR_SPAN
                };
                let name = Span::styled(
                    &x.name,
                    if is_current(i) {
                        Style::default()
                            .fg(Color::Blue)
                            .add_modifier(Modifier::BOLD)
                    } else if is_pointed(i) {
                        text_style.fg(Color::LightBlue)
                    } else {
                        text_style
                    },
                );
                let proxy_type = Span::styled(x.proxy_type.to_string(), Consts::PROXY_TYPE_STYLE);

                let delay_span = x
                    .history
                    .as_ref()
                    .map(|x| {
                        if x.delay > 0 {
                            let style = Self::get_delay_style(x.delay);
                            Span::styled(x.delay.to_string(), style)
                        } else {
                            Span::styled(Consts::NO_LATENCY_SIGN, Consts::NO_LATENCY_STYLE)
                        }
                    })
                    .unwrap_or_else(|| {
                        if !x.proxy_type.is_normal() {
                            Span::raw("")
                        } else {
                            Span::styled(Consts::NO_LATENCY_SIGN, Consts::NO_LATENCY_STYLE)
                        }
                    });
                vec![
                    prefix,
                    Consts::DELIMITER_SPAN.clone(),
                    name,
                    Consts::DELIMITER_SPAN.clone(),
                    proxy_type,
                    Consts::DELIMITER_SPAN.clone(),
                    delay_span,
                ]
                .into()
            });
            ret.extend(lines);
        } else {
            ret.extend(
                self.get_summary_widget()
                    .collect::<Vec<_>>()
                    .chunks(
                        width
                            .saturating_sub(Consts::FOCUSED_INDICATOR_SPAN.width() + 2)
                            .saturating_div(2),
                    )
                    .map(|x| {
                        std::iter::once(if matches!(status, ProxyGroupFocusStatus::Focused) {
                            Consts::FOCUSED_INDICATOR_SPAN
                        } else {
                            Consts::UNFOCUSED_INDICATOR_SPAN
                        })
                        .chain(x.to_owned().into_iter())
                        .collect::<Vec<_>>()
                        .into()
                    }),
            )
        }

        ret
    }

    fn get_delay_style(delay: u64) -> Style {
        match delay {
            0 => Consts::NO_LATENCY_STYLE,
            1..=200 => Consts::LOW_LATENCY_STYLE,
            201..=400 => Consts::MID_LATENCY_STYLE,
            401.. => Consts::HIGH_LATENCY_STYLE,
        }
    }

    fn get_delay_span(delay: u64) -> Span<'static> {
        match delay {
            0 => Consts::NO_LATENCY_SPAN,
            1..=200 => Consts::LOW_LATENCY_SPAN,
            201..=400 => Consts::MID_LATENCY_SPAN,
            401.. => Consts::HIGH_LATENCY_SPAN,
        }
    }
}

impl<'a> Default for ProxyGroup<'a> {
    fn default() -> Self {
        Self {
            members: vec![],
            current: None,
            proxy_type: ProxyType::Selector,
            name: String::new(),
            cursor: 0,
            _life: PhantomData,
        }
    }
}

#[derive(Clone, Debug, Hash)]
pub struct ProxyItem {
    pub name: String,
    pub proxy_type: ProxyType,
    pub history: Option<History>,
    pub udp: bool,
}

impl<'a> From<(&'a str, &'a Proxy)> for ProxyItem {
    fn from(val: (&'a str, &'a Proxy)) -> Self {
        let (name, proxy) = val;
        Self {
            name: name.to_owned(),
            proxy_type: proxy.proxy_type,
            history: proxy.history.get(0).cloned(),
            udp: proxy.udp,
        }
    }
}

#[derive(Clone, Debug, Default)]
pub struct ProxyTree<'a> {
    pub groups: Vec<ProxyGroup<'a>>,
    pub expanded: bool,
    pub cursor: usize,
}

impl<'a> ProxyTree<'a> {
    pub fn toggle(&mut self) {
        self.expanded = !self.expanded
    }

    pub fn merge(&mut self, other: ProxyTree<'a>) {
        if get_hash(&self.groups) == get_hash(&other.groups) {
            return;
        }

        let mut map: HashMap<_, _> =
            FromIterator::from_iter(other.groups.into_iter().map(|x| (x.name.to_owned(), x)));

        for group in self.groups.iter_mut() {
            if let Some(other_group) = map.remove(&group.name) {
                if get_hash(group) == get_hash(&other_group) {
                    continue;
                }
                *group = ProxyGroup {
                    cursor: group.cursor,
                    ..other_group
                }
            }
        }

        for (_, group) in map.into_iter() {
            self.groups.push(group)
        }
    }
}

impl<'a> From<Proxies> for ProxyTree<'a> {
    fn from(val: Proxies) -> Self {
        let mut ret = Self {
            groups: Vec::with_capacity(val.len()),
            ..Default::default()
        };
        for (name, group) in val.groups() {
            let all = group.all.as_ref().expect("ProxyGroup should have member");
            let mut members = Vec::with_capacity(all.len());
            for x in all.iter() {
                let member = (
                    x.as_str(),
                    val.get(x)
                        .to_owned()
                        .expect("Group member should be in all proxies"),
                )
                    .into();
                members.push(member);
            }

            // if group.now.is_some then it must be in all proxies
            // So use map & expect instead of Option#and_then
            let current = group.now.as_ref().map(|name| {
                members
                    .iter()
                    .position(|item: &ProxyItem| &item.name == name)
                    .expect("Group member should be in all proxies")
            });

            ret.groups.push(ProxyGroup {
                _life: PhantomData,
                name: name.to_owned(),
                proxy_type: group.proxy_type,
                cursor: current.unwrap_or_default(),
                current,
                members,
            })
        }
        ret.groups.sort_by_cached_key(|x| x.name.to_owned());
        ret
    }
}

#[derive(Clone, Debug)]
pub struct ProxyTreeWidget<'a> {
    state: &'a ProxyTree<'a>,
    _life: PhantomData<&'a ()>,
}

impl<'a> ProxyTreeWidget<'a> {
    pub fn new(state: &'a ProxyTree<'a>) -> Self {
        Self {
            _life: PhantomData,
            state,
        }
    }
}

impl<'a> Widget for ProxyTreeWidget<'a> {
    fn render(self, area: tui::layout::Rect, buf: &mut tui::buffer::Buffer) {
        let cursor = &self.state.cursor;
        let skip = if self.state.expanded {
            *cursor
        } else {
            cursor.saturating_sub(2)
        };
        let text = self
            .state
            .groups
            .iter()
            .skip(skip)
            .enumerate()
            .map(|(i, x)| {
                x.get_widget(
                    area.width as usize,
                    match (self.state.expanded, *cursor == i + skip) {
                        (true, true) => ProxyGroupFocusStatus::Expanded,
                        (false, true) => ProxyGroupFocusStatus::Focused,
                        _ => ProxyGroupFocusStatus::None,
                    },
                )
            })
            .reduce(|mut a, b| {
                a.extend(b);
                a
            })
            .unwrap_or_default()
            .into_iter()
            .take(area.height as usize)
            .collect::<Vec<_>>();

        let block = if self.state.expanded {
            get_focused_block("Proxies")
        } else {
            get_block("Proxies")
        };

        let inner = block.inner(area);
        // if area.height > 12 {
        //     inner.x += 1;
        //     inner.width -= 2;
        //     inner.y += 1;
        //     inner.height -= 2;
        // }
        block.render(area, buf);

        Paragraph::new(text).render(inner, buf);
    }
}
