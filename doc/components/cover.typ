#import "typography.typ": 字体, 字号

#let cover(
  logo: "",
  title: "",
  institute: "",
  year: datetime.today().year(),
  month: datetime.today().month(),
) = {
  align(center)[
    // 添加校徽图片（在顶部居中显示）
    #if logo != "" {
      image(logo, width: 3cm)  // 调整width参数控制图片大小
    }
    #let space_scale_ratio = 1.2

    #v(字号.小四 * 3 * space_scale_ratio)

    #text(size: 字号.小一, font: 字体.宋体, weight: "bold")[*操作系统内核设计*]

    #v(字号.小四 * 2 * space_scale_ratio)

    #text(size: 字号.二号, font: 字体.黑体)[#title]

    #v(字号.小四 * 2 * space_scale_ratio)

    #v(字号.小四 * 1 * space_scale_ratio)
    #v(字号.二号 * 2 * space_scale_ratio)

    #v(字号.小二 * 2 * space_scale_ratio)
    #v(字号.小四 * 6 * space_scale_ratio)

    #align(center)[
      #text(size: 字号.小二, font: 字体.楷体, weight: "bold")[#institute]

      #text(size: 字号.小二, font: 字体.宋体, weight: "bold")[
        #[#year]年#[#month]月
      ]
    ]
  ]
}
