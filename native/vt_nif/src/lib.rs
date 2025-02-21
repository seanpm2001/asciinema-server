use avt::Vt;
use rustler::{Atom, Binary, Encoder, Env, Error, NifResult, ResourceArc, Term};
use std::sync::RwLock;

mod atoms {
    rustler::atoms! {
        ok,
        error,
        invalid_size,
    }
}

pub struct VtResource {
    vt: RwLock<Vt>,
}

fn load(env: Env, _info: Term) -> bool {
    rustler::resource!(VtResource, env);

    true
}

#[rustler::nif]
fn new(w: usize, h: usize) -> NifResult<(Atom, ResourceArc<VtResource>)> {
    if w > 0 && h > 0 {
        let vt = Vt::new(w, h);
        let resource = ResourceArc::new(VtResource {
            vt: RwLock::new(vt),
        });

        Ok((atoms::ok(), resource))
    } else {
        Err(Error::Term(Box::new(atoms::invalid_size())))
    }
}

#[rustler::nif]
fn feed(resource: ResourceArc<VtResource>, input: Binary) -> NifResult<Option<(usize, usize)>> {
    let mut vt = convert_err(resource.vt.write(), "rw_lock")?;
    let (_, resized) = vt.feed_str(&String::from_utf8_lossy(&input));

    if resized {
        Ok(Some(vt.size()))
    } else {
        Ok(None)
    }
}

#[rustler::nif]
fn dump(resource: ResourceArc<VtResource>) -> NifResult<String> {
    let vt = convert_err(resource.vt.read(), "rw_lock")?;

    Ok(vt.dump())
}

#[rustler::nif]
fn dump_screen(env: Env, resource: ResourceArc<VtResource>) -> NifResult<(Atom, Term)> {
    let vt = convert_err(resource.vt.read(), "rw_lock")?;

    let lines = vt
        .view()
        .iter()
        .map(|line| {
            line.segments()
                .map(|segment| segment_to_term(segment, env))
                .collect::<Vec<_>>()
        })
        .collect::<Vec<_>>();

    let (col, row, visible) = vt.cursor();
    let cursor = if visible { Some((col, row)) } else { None };

    Ok((atoms::ok(), (lines, cursor).encode(env)))
}

fn segment_to_term(segment: avt::Segment, env: Env) -> Term {
    let text = segment.text();
    let mut pairs: Vec<(String, Term)> = Vec::new();

    match segment.foreground() {
        Some(avt::Color::Indexed(c)) => {
            pairs.push(("fg".to_owned(), c.encode(env)));
        }

        Some(avt::Color::RGB(c)) => {
            let c = format!("rgb({},{},{})", c.r, c.g, c.b);
            pairs.push(("fg".to_owned(), c.encode(env)));
        }

        None => (),
    }

    match segment.background() {
        Some(avt::Color::Indexed(c)) => {
            pairs.push(("bg".to_owned(), c.encode(env)));
        }

        Some(avt::Color::RGB(c)) => {
            let c = format!("rgb({},{},{})", c.r, c.g, c.b);
            pairs.push(("bg".to_owned(), c.encode(env)));
        }

        None => (),
    }

    if segment.is_bold() {
        pairs.push(("bold".to_owned(), true.encode(env)));
    }

    if segment.is_faint() {
        pairs.push(("faint".to_owned(), true.encode(env)));
    }

    if segment.is_italic() {
        pairs.push(("italic".to_owned(), true.encode(env)));
    }

    if segment.is_underline() {
        pairs.push(("underline".to_owned(), true.encode(env)));
    }

    if segment.is_strikethrough() {
        pairs.push(("strikethrough".to_owned(), true.encode(env)));
    }

    if segment.is_blink() {
        pairs.push(("blink".to_owned(), true.encode(env)));
    }

    if segment.is_inverse() {
        pairs.push(("inverse".to_owned(), true.encode(env)));
    }

    let attrs = Term::map_from_pairs(env, &pairs).unwrap();

    (text, attrs).encode(env)
}

fn convert_err<T, E>(result: Result<T, E>, error: &'static str) -> Result<T, Error> {
    match result {
        Ok(value) => Ok(value),
        Err(_) => Err(Error::RaiseAtom(error)),
    }
}

rustler::init!(
    "Elixir.Asciinema.Vt",
    [new, feed, dump, dump_screen],
    load = load
);
