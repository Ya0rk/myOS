#import "typography.typ": 字体

#let outline-page() = [
  #set par(first-line-indent: 0em)

  #[
    #show heading: none
    #heading([目录], level: 1, outlined: false)
  ]

  #show outline.entry: (entry, label: <outline-page-modified-entry>) => {

    if entry.at("label", default: none) == label {
      entry // prevent infinite recursion
    } else {
      let fields = entry.fields()
      if entry.level == 1 {
        fields.body = [#text(font: 字体.黑体)[#fields.body]]
      }
      [#outline.entry(..fields.values()) #label]
    }
  }

  #outline(title: align(center)[目录], indent: n => [#h(1em)] * n)
]