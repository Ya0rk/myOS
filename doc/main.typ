#import "conf.typ": doc, preface, main, project
#import "components/cover.typ": cover
#import "components/figure.typ": algorithm-figure, code-figure
#import "components/outline.typ": outline-page
#import "@preview/lovelace:0.2.0": *

#show: doc

#set text(lang: "zh", region: "cn")

#show: project.with(
  projectname: "Del0n1x",
  teamname: "Del0n1x",
  teammates: ("姚俊杰", "卢家鼎", "林顺喆"),
  teachers: ("夏文", "仇洁婷"),
  date: (2025, 6),
  logopath: "./content/assets/hitsz-logo.jpg"
)


#show: preface.with(title: "Del0n1x")

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
