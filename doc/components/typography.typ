#let 字号 = (
  一英寸: 72pt,
  大特号: 63pt,
  特号: 54pt,
  初号: 42pt,
  小初: 36pt,
  一号: 26pt,
  小一: 24pt,
  二号: 22pt,
  小二: 18pt,
  三号: 16pt,
  小三: 15pt,
  四号: 14pt,
  小四: 12pt,
  五号: 10.5pt,
  小五: 9pt,
  六号: 7.5pt,
  小六: 6.5pt,
  七号: 5.5pt,
  八号: 5pt,
)

#let 字体 = (
  宋体: ("Times New Roman", "SimSun"),
  楷体: ("Times New Roman", "KaiTi"),
  黑体: ("Times New Roman", "Noto Sans CJK SC", "SimHei"),
  代码: ("Consolas", "Courier New", "KaiTi"),
)

#let special-chapter-format-heading(it: none, font: none, size: none, weight: "regular") = {
  set text(font: font, size: size)

  text(weight: weight)[
    #if it != none {
      it.body
    }
  ]
  v(0.5em)
}

#let main-format-heading(it: none, font: none, size: none, weight: "regular") = {
  set text(font: font, size: size)

  text(weight: weight)[
    #counter(heading).display()
    #if it != none {
      it.body
    }
  ]
  v(0.5em)
}

#let heading-numbering(..nums) = {
  let nums-vec = nums.pos()

  if nums-vec.len() == 1 [
    #numbering("第 1 章", ..nums-vec) #h(0.75em)
  ] else [
    #numbering("1.1", ..nums-vec) #h(0.75em)
  ]
}
