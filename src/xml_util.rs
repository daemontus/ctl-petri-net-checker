use std::io::Read;
use std::fs::File;
use std::io::BufReader;
use xml::reader::{EventReader, XmlEvent};
use xml::reader::XmlEvent::*;

/// Parse an XML file using the given function
pub fn parse_file<F, R>(name: &String, action: F) -> R
    where F: Fn(&mut EventReader<BufReader<File>>) -> R {
    let file = File::open(name).unwrap();
    let file = BufReader::new(file);
    let mut parser = EventReader::new(file);
    action(&mut parser)
}

///Read opening tag, then perform action and finally read closing tag
pub fn inside<T, R, F>(tag: &str, parser: &mut EventReader<T>, action: F) -> R
    where T: Read, F: Fn(&mut EventReader<T>) -> R {
    expect_tag_open(tag, parser);
    let result = action(parser);
    expect_tag_close(tag, parser);
    result
}

///Consume parser events until either end tag is encountered or
///the action tag is encountered, in which case the action is called.
pub fn find_until<T, R, F>(tag: &str, end: &str, parser: &mut EventReader<T>, action: F) -> Option<R>
    where T: Read, F: Fn(&mut EventReader<T>) -> R {
    drop_until(parser, |p, e| {
        //first see if we are at the end
        expect_end(e, end).map_or_else(|| {
            //if not, try reading the start tag
            expect_start(e, tag).map(|_| {
                let result = action(p);
                expect_tag_close(tag, p);
                Some(result)
            })
        }, |_| Some(None))  //if end, stop dropping but return not found
    }).unwrap_or_else(|| {
        panic!("Can't find <{:?}> or </{:?}>", tag, end);
    })
}

///Expect opening tag and then collect items until closing tag is encountered
///Action will be called
pub fn collect_inside<T, F, R>(tag: &str, parser: &mut EventReader<T>, action: F) -> Vec<R>
    where T: Read, F: Fn(&mut EventReader<T>, &XmlEvent) -> Option<R> {
    expect_tag_open(tag, parser);
    //note: We can't really use closures here because we would have to lock the vector
    let mut results = Vec::new();
    loop {
        match parser.next() {
            Ok(EndDocument) => panic!("Unexpected end of document, waiting for </{}>", tag),
            Ok(event) => {
                if expect_end(&event, tag).is_some() {
                    return results;
                } else if let Some(item) = action(parser, &event) {
                    results.push(item);
                }
            }
            e => panic!("Error reading document: {:?}", e),
        }
    }
}

///Consume parser events until a specific opening tag is encountered
pub fn expect_tag_open<T: Read>(tag: &str, parser: &mut EventReader<T>) {
    if let None = drop_until(parser, |_, e| expect_start(e, tag)) {
        panic!("Can't find end tag <{}>", tag);
    }
}

///Consume parser events until a specific closing tag is encountered
pub fn expect_tag_close<T: Read>(tag: &str, parser: &mut EventReader<T>) {
    if let None = drop_until(parser, |_, e| expect_end(e, tag)) {
        panic!("Can't find end tag </{}>", tag);
    }
}

///Comsume parser evnets until the next opening tag is encountered
pub fn next_tag_open<T: Read>(parser: &mut EventReader<T>) -> String {
    drop_until(parser, |_, e| match_start_name(e)).unwrap_or_else( ||
        panic!("Can't find next start tag")
    )
}

///Comsume parser events until the next text event is encountered
pub fn next_text<T: Read>(parser: &mut EventReader<T>) -> String {
    drop_until(parser, |_, e| match_text(e)).unwrap_or_else( ||
        panic!("Can't find next start tag")
    )
}

///Consume events from parser until one that passes the test is encountered
pub fn drop_until<T, F, R>(parser: &mut EventReader<T>, test: F) -> Option<R>
    where T : Read, F : Fn(&mut EventReader<T>, &XmlEvent) -> Option<R> {
    loop {
        match parser.next() {
            Ok(EndDocument) => return None,
            Ok(event) => {
                if let Some(result) = test(parser, &event) {
                    return Some(result);
                }
            }
            e => panic!("Error reading document: {:?}", e),
        }
    }
}

///Get name of the starting tag if avaiable
pub fn match_start_name(event: &XmlEvent) -> Option<String> {
    match event {
        &StartElement { ref name, .. } => Some(name.local_name.clone()),
        _ => None
    }
}

///Get name of the ending tag if avaiable
pub fn match_text(event: &XmlEvent) -> Option<String> {
    match event {
        &Characters(ref text) => Some(text.clone()),
        _ => None
    }
}

///Return some if event is specified starting tag
pub fn expect_start(event: &XmlEvent, tag: &str) -> Option<()> {
    match event {
        &StartElement { ref name, .. } if name.local_name == tag => Some(()),
        _ => None
    }
}

///Return some if event is specified ending tag
pub fn expect_end(event: &XmlEvent, tag: &str) -> Option<()> {
    match event {
        &EndElement { ref name, .. } if name.local_name == tag => Some(()),
        _ => None
    }
}
