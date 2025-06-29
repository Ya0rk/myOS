#import "components/typography.typ": 字体, 字号, main-format-heading, special-chapter-format-heading, heading-numbering
#import "@preview/cuti:0.2.1": show-cn-fakebold
#import "@preview/i-figured:0.2.4": show-figure, reset-counters, show-equation

#let doc(content) = {
  set page(
    paper: "a4",
    margin: (top: 3.8cm, left: 3cm, right: 3cm, bottom: 3cm),
  )

  show: show-cn-fakebold

  content
}

#let preface(content, title: "") = {
  set page(
    header: {
      [
        #set align(center)
        #set par(leading: 0em)
        #text(font: 字体.宋体, size: 字号.小五, baseline: 8.5pt)[
          操作系统内核设计 - #title
        ]
        #line(length: 100%, stroke: 2.2pt)
        #v(2.2pt, weak: true)
        #line(length: 100%, stroke: 0.6pt)
      ]
    },
    header-ascent: 15%,
  )

  set page(numbering: "I")

  set page(
    footer: context [
      #align(center)[
        #counter(page).display("- I -")
      ]
    ],
    footer-descent: 15%,
  )

  counter(page).update(1)


  show heading: it => {
    set par(first-line-indent: 0em)

    if it.level == 1 {
      align(center)[
        #v(1em)
        #special-chapter-format-heading(it: it, font: 字体.黑体, size: 字号.小二)
        #v(.3em)
      ]
    } else {
      it
    }
  }


  set par(first-line-indent: 2em, leading: 1em, justify: true)

  set text(font: 字体.宋体, size: 字号.小四)

  content
}

#let main(content) = {
  set page(numbering: "1")

  set page(footer: context [
    #align(center)[
      #counter(page).display("- 1 -")
    ]
  ])

  counter(page).update(1)

  set heading(numbering: heading-numbering)

  show heading: it => {
    set par(first-line-indent: 0em)

    if it.level == 1 {
      align(center)[
        #v(1em)
        #main-format-heading(it: it, font: 字体.黑体, size: 字号.小二)
        #v(.3em)
      ]
    } else if it.level == 2 {
      main-format-heading(it: it, font: 字体.黑体, size: 字号.小三)
    } else if it.level >= 3 {
      main-format-heading(it: it, font: 字体.黑体, size: 字号.小四)
    }
  }

  show heading: reset-counters.with(extra-kinds: ("algorithm",))
  show figure: show-figure.with(numbering: "1-1", extra-prefixes: ("algorithm": "algo:"))
  show figure.where(kind: table): set figure.caption(position: top)
  show figure.where(kind: "algorithm"): set figure.caption(position: top)

  show raw.where(block: false): box.with(
    fill: rgb("#fafafa"),
    inset: (x: 3pt, y: 0pt),
    outset: (y: 3pt),
    radius: 2pt,
  )

  show raw.where(block: false): text.with(
    font: 字体.代码,
    size: 10.5pt,
  )
  show raw.where(block: true): block.with(
    fill: rgb("#fafafa"),
    inset: 8pt,
    radius: 4pt,
    width: 100%,
  )
  show raw.where(block: true): text.with(
    font: 字体.代码,
    size: 10.5pt,
  )

  show math.equation: show-equation.with(numbering: "(1-1)")

  show ref: it => {
    let eq = math.equation
    let el = it.element
    if el != none and el.func() == eq {
      // Override equation references.
      numbering(
        el.numbering,
        ..counter(eq).at(el.location()),
      )
    } else {
      // Other references as usual.
      it
    }
  }

  content
}
#import "./components/typography.typ": 字体, 字号
#let project(
  projectname: "",
  teamname: "",
  teammates: (),
  teachers: (),
  date: (1926, 8, 17),
  logopath: "",
  body,
) = {
  // 封面
  align(center)[
    // hust logo
    #v(30pt)

    #image(logopath, width: 100%)

    #v(50pt)

    #text(
      size: 36pt,
      font: 字体.黑体,
      weight: "bold"
    )[#projectname]

    #v(40pt)

    #text(
      font: 字体.黑体,
      size: 22pt,
    )[
      设计文档
    ]

    #v(100pt)

    #let info_value(body) = {
      rect(
        width: 100%,
        inset: 2pt,
        stroke: (
          bottom: 1pt + black
        ),
        text(
          font: 字体.黑体,
          size: 16pt,
          bottom-edge: "descender"
        )[
          #body
        ]
      ) 
    }
    
    #let info_key(body) = {
      rect(width: 100%, inset: 2pt, 
       stroke: none,
       text(
        font: 字体.黑体,
        size: 16pt,
        body
      ))
    }

    #grid(
      columns: (70pt, 180pt),
      rows: (40pt, 40pt),
      gutter: 3pt,
      info_key("参赛队名"),
      info_value(teamname),
      info_key("队伍成员"),
      info_value(teammates.join("、")),
      info_key("指导老师"),
      info_value(teachers.join("、")),
    )

    #v(40pt)
    #text(
      font: 字体.黑体,
      size: 16pt,
    )[
      #date.at(0) 年 #date.at(1) 月
    ]
    // #pagebreak()
  ]


  body
}