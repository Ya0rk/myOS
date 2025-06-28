#let algorithm-figure(content, caption: none, supplement: [算法], label-name: "", breakable: true) = {
  block(stroke: rgb("#0000"))[
    #let new-label = label(label-name)
    #figure(
      [],
      kind: "algorithm",
      supplement: supplement,
    ) #new-label
    #v(-1.25em)

    #context {
      let heading-number = counter(heading).get().at(0)
      let _prefix = "i-figured-"
      let algo-kind = "algorithm"
      let prefix-alog-number = counter(figure.where(kind: _prefix + repr(algo-kind))).get().at(0)
      let numbers = (heading-number, prefix-alog-number)

      block(
        stroke: (y: 1.3pt),
        inset: 0pt,
        width: 100%,
        {
          set align(left)
          block(
            inset: (y: 5pt),
            width: 100%,
            stroke: (bottom: .8pt),
            {
              strong({
                supplement
                numbering("1-1", ..numbers)
                [: ]
              })
              caption
            },
          )
          v(-1em)
          block(content)
          v(.5em)
        },
      )
    }
  ]
}
