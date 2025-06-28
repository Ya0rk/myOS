#import "conf.typ": doc, preface, main
#import "components/cover.typ": cover
#import "components/figure.typ": algorithm-figure, code-figure
#import "components/outline.typ": outline-page
#import "@preview/lovelace:0.2.0": *

#show: doc

#set text(lang: "zh", region: "cn")

#cover(
  title: "MinotaurOS",
  institute: "哈尔滨工业大学",
)

#show: preface.with(title: "MinotaurOS")

#outline-page()

#show: main

#include "content/00-brief.typ"
#include "content/01-process.typ"
#include "content/88-trap.typ"
#include "content/02-mem.typ"
#include "content/03-fs.typ"
#include "content/04-ipc.typ"
#include "content/05-time.typ"
#include "content/06-net.typ"
#include "content/07-device.typ"
#include "content/08-hal.typ"
#include "content/99-prospect.typ"
