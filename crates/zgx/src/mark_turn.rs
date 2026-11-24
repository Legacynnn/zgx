//! A text area's line marks turning (`LineMark::rotate`: a toggle's chevron
//! opening): the app says the angle a mark ends at, and the mark eases there
//! over `motion.duration.fast`, easing out, as the web's `transition`. The
//! host wakes for each step with the scroll reveals' (`scroll_reveal`, whose
//! deadline the turns share: a window with neither asks nothing more), so a
//! turn costs a few small frames and no frame loop, and once every turn
//! settles nothing is scheduled: an idle window does no work.
use crate::{scene::NodeId, time::Instant, widgets::Ui};
use std::{collections::HashMap, ops::Range, time::Duration};

/// How often a turning mark steps: as often as a 120 Hz display draws.
pub(crate) const STEP: Duration = Duration::from_micros(8_333);

/// One mark's turn (`LineMark::rotate`): from what angle, to what, since
/// when, in degrees.
#[derive(Clone, Copy, Debug)]
struct Turn {
    from: f32,
    to: f32,
    start: Instant,
}
impl Turn {
    /// Where it stands at `now`, turning over `duration`, and whether it is
    /// still turning.
    fn angle(&self, now: Instant, duration: Duration) -> (f32, bool) {
        if self.from == self.to || duration.is_zero() {
            return (self.to, false);
        }
        let t = (now.saturating_duration_since(self.start).as_secs_f32() / duration.as_secs_f32())
            .clamp(0., 1.);
        let eased = 1. - (1. - t).powi(3);
        (self.from + (self.to - self.from) * eased, t < 1.)
    }
    /// Turn toward `to` from where it stands at `now`: a change mid-turn
    /// starts from there, not from its old end.
    fn toward(&mut self, to: f32, now: Instant, duration: Duration) {
        if to != self.to {
            let (here, _) = self.angle(now, duration);
            *self = Self {
                from: here,
                to,
                start: now,
            };
        }
    }
}

/// A text area's marks' turns, by line and place among its marks: empty
/// until one turns.
#[derive(Default)]
pub(crate) struct Turns {
    each: HashMap<(usize, usize), Turn>,
    /// A mark has turned: its nodes' transforms are kept from then on.
    pub(crate) any: bool,
    /// A mark is still turning, as of the last draw.
    pub(crate) moving: bool,
    /// The lines whose marks the last draw showed: a mark turned on one of
    /// them was showing unturned, and eases from there; one first shown
    /// turned (scrolled to, or at the first draw) shows at its angle.
    shown: Range<usize>,
    /// How many lines the text had: with another count its lines moved, and
    /// every mark shows at its angle.
    lines: usize,
}
impl Turns {
    /// A draw of a text of `lines` lines starts.
    pub(crate) fn begin(&mut self, lines: usize) {
        self.moving = false;
        if self.lines != lines {
            self.lines = lines;
            self.shown = 0..0;
            self.each.clear();
        }
    }
    /// The angle the mark `at` (its line, its place) given `rotate` shows.
    pub(crate) fn angle(
        &mut self,
        at: (usize, usize),
        rotate: f32,
        now: Instant,
        duration: Duration,
    ) -> f32 {
        if rotate == 0. && !self.any {
            return 0.;
        }
        self.any = true;
        let from = if self.shown.contains(&at.0) {
            0.
        } else {
            rotate
        };
        let turn = self.each.entry(at).or_insert(Turn {
            from,
            to: from,
            start: now,
        });
        turn.toward(rotate, now, duration);
        let (angle, moving) = turn.angle(now, duration);
        self.moving |= moving;
        angle
    }
    /// The draw showed the marks of lines `shown`: the rest's turns go.
    pub(crate) fn end(&mut self, shown: Range<usize>) {
        self.each.retain(|(line, _), _| shown.contains(line));
        self.shown = shown;
    }
}

/// Draw again each text area whose marks were turning: each draw steps its
/// marks, and asks for another only while one still turns.
pub(crate) fn step(ui: &Ui, nodes: Vec<NodeId>) {
    for node in nodes {
        if let Some(editor) = ui.editor(node) {
            editor.refresh();
        }
    }
}
