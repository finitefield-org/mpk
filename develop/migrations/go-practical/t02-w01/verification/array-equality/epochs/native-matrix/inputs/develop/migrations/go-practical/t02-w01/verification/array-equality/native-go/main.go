package main
import("encoding/json";"os")
type Input struct {ID string `json:"id"`; Length int `json:"length"`; Left,Right []int}
type Output struct {ID string `json:"id"`; Length int `json:"length"`; Equal bool `json:"equal"`; NotEqual bool `json:"not_equal"`}
func equal0(a,b [0]byte)bool{return a==b}
func notEqual0(a,b [0]byte)bool{return a!=b}
func equal1(a,b [1]byte)bool{return a==b}
func notEqual1(a,b [1]byte)bool{return a!=b}
func equal2(a,b [2]byte)bool{return a==b}
func notEqual2(a,b [2]byte)bool{return a!=b}
func equal3(a,b [3]byte)bool{return a==b}
func notEqual3(a,b [3]byte)bool{return a!=b}
func equal4(a,b [4]byte)bool{return a==b}
func notEqual4(a,b [4]byte)bool{return a!=b}
func equal5(a,b [5]byte)bool{return a==b}
func notEqual5(a,b [5]byte)bool{return a!=b}
func equal6(a,b [6]byte)bool{return a==b}
func notEqual6(a,b [6]byte)bool{return a!=b}
func equal7(a,b [7]byte)bool{return a==b}
func notEqual7(a,b [7]byte)bool{return a!=b}
func equal8(a,b [8]byte)bool{return a==b}
func notEqual8(a,b [8]byte)bool{return a!=b}
func equal9(a,b [9]byte)bool{return a==b}
func notEqual9(a,b [9]byte)bool{return a!=b}
func equal10(a,b [10]byte)bool{return a==b}
func notEqual10(a,b [10]byte)bool{return a!=b}
func equal11(a,b [11]byte)bool{return a==b}
func notEqual11(a,b [11]byte)bool{return a!=b}
func equal12(a,b [12]byte)bool{return a==b}
func notEqual12(a,b [12]byte)bool{return a!=b}
func equal13(a,b [13]byte)bool{return a==b}
func notEqual13(a,b [13]byte)bool{return a!=b}
func equal14(a,b [14]byte)bool{return a==b}
func notEqual14(a,b [14]byte)bool{return a!=b}
func equal15(a,b [15]byte)bool{return a==b}
func notEqual15(a,b [15]byte)bool{return a!=b}
func equal16(a,b [16]byte)bool{return a==b}
func notEqual16(a,b [16]byte)bool{return a!=b}
func equal17(a,b [17]byte)bool{return a==b}
func notEqual17(a,b [17]byte)bool{return a!=b}
func equal18(a,b [18]byte)bool{return a==b}
func notEqual18(a,b [18]byte)bool{return a!=b}
func equal19(a,b [19]byte)bool{return a==b}
func notEqual19(a,b [19]byte)bool{return a!=b}
func equal20(a,b [20]byte)bool{return a==b}
func notEqual20(a,b [20]byte)bool{return a!=b}
func equal21(a,b [21]byte)bool{return a==b}
func notEqual21(a,b [21]byte)bool{return a!=b}
func equal22(a,b [22]byte)bool{return a==b}
func notEqual22(a,b [22]byte)bool{return a!=b}
func equal23(a,b [23]byte)bool{return a==b}
func notEqual23(a,b [23]byte)bool{return a!=b}
func equal24(a,b [24]byte)bool{return a==b}
func notEqual24(a,b [24]byte)bool{return a!=b}
func equal25(a,b [25]byte)bool{return a==b}
func notEqual25(a,b [25]byte)bool{return a!=b}
func equal26(a,b [26]byte)bool{return a==b}
func notEqual26(a,b [26]byte)bool{return a!=b}
func equal27(a,b [27]byte)bool{return a==b}
func notEqual27(a,b [27]byte)bool{return a!=b}
func equal28(a,b [28]byte)bool{return a==b}
func notEqual28(a,b [28]byte)bool{return a!=b}
func equal29(a,b [29]byte)bool{return a==b}
func notEqual29(a,b [29]byte)bool{return a!=b}
func equal30(a,b [30]byte)bool{return a==b}
func notEqual30(a,b [30]byte)bool{return a!=b}
func equal31(a,b [31]byte)bool{return a==b}
func notEqual31(a,b [31]byte)bool{return a!=b}
func equal32(a,b [32]byte)bool{return a==b}
func notEqual32(a,b [32]byte)bool{return a!=b}
func equal33(a,b [33]byte)bool{return a==b}
func notEqual33(a,b [33]byte)bool{return a!=b}
func equal34(a,b [34]byte)bool{return a==b}
func notEqual34(a,b [34]byte)bool{return a!=b}
func equal35(a,b [35]byte)bool{return a==b}
func notEqual35(a,b [35]byte)bool{return a!=b}
func equal36(a,b [36]byte)bool{return a==b}
func notEqual36(a,b [36]byte)bool{return a!=b}
func equal37(a,b [37]byte)bool{return a==b}
func notEqual37(a,b [37]byte)bool{return a!=b}
func equal38(a,b [38]byte)bool{return a==b}
func notEqual38(a,b [38]byte)bool{return a!=b}
func equal39(a,b [39]byte)bool{return a==b}
func notEqual39(a,b [39]byte)bool{return a!=b}
func equal40(a,b [40]byte)bool{return a==b}
func notEqual40(a,b [40]byte)bool{return a!=b}
func equal41(a,b [41]byte)bool{return a==b}
func notEqual41(a,b [41]byte)bool{return a!=b}
func equal42(a,b [42]byte)bool{return a==b}
func notEqual42(a,b [42]byte)bool{return a!=b}
func equal43(a,b [43]byte)bool{return a==b}
func notEqual43(a,b [43]byte)bool{return a!=b}
func equal44(a,b [44]byte)bool{return a==b}
func notEqual44(a,b [44]byte)bool{return a!=b}
func equal45(a,b [45]byte)bool{return a==b}
func notEqual45(a,b [45]byte)bool{return a!=b}
func equal46(a,b [46]byte)bool{return a==b}
func notEqual46(a,b [46]byte)bool{return a!=b}
func equal47(a,b [47]byte)bool{return a==b}
func notEqual47(a,b [47]byte)bool{return a!=b}
func equal48(a,b [48]byte)bool{return a==b}
func notEqual48(a,b [48]byte)bool{return a!=b}
func equal49(a,b [49]byte)bool{return a==b}
func notEqual49(a,b [49]byte)bool{return a!=b}
func equal50(a,b [50]byte)bool{return a==b}
func notEqual50(a,b [50]byte)bool{return a!=b}
func equal51(a,b [51]byte)bool{return a==b}
func notEqual51(a,b [51]byte)bool{return a!=b}
func equal52(a,b [52]byte)bool{return a==b}
func notEqual52(a,b [52]byte)bool{return a!=b}
func equal53(a,b [53]byte)bool{return a==b}
func notEqual53(a,b [53]byte)bool{return a!=b}
func equal54(a,b [54]byte)bool{return a==b}
func notEqual54(a,b [54]byte)bool{return a!=b}
func equal55(a,b [55]byte)bool{return a==b}
func notEqual55(a,b [55]byte)bool{return a!=b}
func equal56(a,b [56]byte)bool{return a==b}
func notEqual56(a,b [56]byte)bool{return a!=b}
func equal57(a,b [57]byte)bool{return a==b}
func notEqual57(a,b [57]byte)bool{return a!=b}
func equal58(a,b [58]byte)bool{return a==b}
func notEqual58(a,b [58]byte)bool{return a!=b}
func equal59(a,b [59]byte)bool{return a==b}
func notEqual59(a,b [59]byte)bool{return a!=b}
func equal60(a,b [60]byte)bool{return a==b}
func notEqual60(a,b [60]byte)bool{return a!=b}
func equal61(a,b [61]byte)bool{return a==b}
func notEqual61(a,b [61]byte)bool{return a!=b}
func equal62(a,b [62]byte)bool{return a==b}
func notEqual62(a,b [62]byte)bool{return a!=b}
func equal63(a,b [63]byte)bool{return a==b}
func notEqual63(a,b [63]byte)bool{return a!=b}
func equal64(a,b [64]byte)bool{return a==b}
func notEqual64(a,b [64]byte)bool{return a!=b}
func equal65(a,b [65]byte)bool{return a==b}
func notEqual65(a,b [65]byte)bool{return a!=b}
func equal66(a,b [66]byte)bool{return a==b}
func notEqual66(a,b [66]byte)bool{return a!=b}
func equal67(a,b [67]byte)bool{return a==b}
func notEqual67(a,b [67]byte)bool{return a!=b}
func equal68(a,b [68]byte)bool{return a==b}
func notEqual68(a,b [68]byte)bool{return a!=b}
func equal69(a,b [69]byte)bool{return a==b}
func notEqual69(a,b [69]byte)bool{return a!=b}
func equal70(a,b [70]byte)bool{return a==b}
func notEqual70(a,b [70]byte)bool{return a!=b}
func equal71(a,b [71]byte)bool{return a==b}
func notEqual71(a,b [71]byte)bool{return a!=b}
func equal72(a,b [72]byte)bool{return a==b}
func notEqual72(a,b [72]byte)bool{return a!=b}
func equal73(a,b [73]byte)bool{return a==b}
func notEqual73(a,b [73]byte)bool{return a!=b}
func equal74(a,b [74]byte)bool{return a==b}
func notEqual74(a,b [74]byte)bool{return a!=b}
func equal75(a,b [75]byte)bool{return a==b}
func notEqual75(a,b [75]byte)bool{return a!=b}
func equal76(a,b [76]byte)bool{return a==b}
func notEqual76(a,b [76]byte)bool{return a!=b}
func equal77(a,b [77]byte)bool{return a==b}
func notEqual77(a,b [77]byte)bool{return a!=b}
func equal78(a,b [78]byte)bool{return a==b}
func notEqual78(a,b [78]byte)bool{return a!=b}
func equal79(a,b [79]byte)bool{return a==b}
func notEqual79(a,b [79]byte)bool{return a!=b}
func equal80(a,b [80]byte)bool{return a==b}
func notEqual80(a,b [80]byte)bool{return a!=b}
func equal81(a,b [81]byte)bool{return a==b}
func notEqual81(a,b [81]byte)bool{return a!=b}
func equal82(a,b [82]byte)bool{return a==b}
func notEqual82(a,b [82]byte)bool{return a!=b}
func equal83(a,b [83]byte)bool{return a==b}
func notEqual83(a,b [83]byte)bool{return a!=b}
func equal84(a,b [84]byte)bool{return a==b}
func notEqual84(a,b [84]byte)bool{return a!=b}
func equal85(a,b [85]byte)bool{return a==b}
func notEqual85(a,b [85]byte)bool{return a!=b}
func equal86(a,b [86]byte)bool{return a==b}
func notEqual86(a,b [86]byte)bool{return a!=b}
func equal87(a,b [87]byte)bool{return a==b}
func notEqual87(a,b [87]byte)bool{return a!=b}
func equal88(a,b [88]byte)bool{return a==b}
func notEqual88(a,b [88]byte)bool{return a!=b}
func equal89(a,b [89]byte)bool{return a==b}
func notEqual89(a,b [89]byte)bool{return a!=b}
func equal90(a,b [90]byte)bool{return a==b}
func notEqual90(a,b [90]byte)bool{return a!=b}
func equal91(a,b [91]byte)bool{return a==b}
func notEqual91(a,b [91]byte)bool{return a!=b}
func equal92(a,b [92]byte)bool{return a==b}
func notEqual92(a,b [92]byte)bool{return a!=b}
func equal93(a,b [93]byte)bool{return a==b}
func notEqual93(a,b [93]byte)bool{return a!=b}
func equal94(a,b [94]byte)bool{return a==b}
func notEqual94(a,b [94]byte)bool{return a!=b}
func equal95(a,b [95]byte)bool{return a==b}
func notEqual95(a,b [95]byte)bool{return a!=b}
func equal96(a,b [96]byte)bool{return a==b}
func notEqual96(a,b [96]byte)bool{return a!=b}
func equal97(a,b [97]byte)bool{return a==b}
func notEqual97(a,b [97]byte)bool{return a!=b}
func equal98(a,b [98]byte)bool{return a==b}
func notEqual98(a,b [98]byte)bool{return a!=b}
func equal99(a,b [99]byte)bool{return a==b}
func notEqual99(a,b [99]byte)bool{return a!=b}
func equal100(a,b [100]byte)bool{return a==b}
func notEqual100(a,b [100]byte)bool{return a!=b}
func equal101(a,b [101]byte)bool{return a==b}
func notEqual101(a,b [101]byte)bool{return a!=b}
func equal102(a,b [102]byte)bool{return a==b}
func notEqual102(a,b [102]byte)bool{return a!=b}
func equal103(a,b [103]byte)bool{return a==b}
func notEqual103(a,b [103]byte)bool{return a!=b}
func equal104(a,b [104]byte)bool{return a==b}
func notEqual104(a,b [104]byte)bool{return a!=b}
func equal105(a,b [105]byte)bool{return a==b}
func notEqual105(a,b [105]byte)bool{return a!=b}
func equal106(a,b [106]byte)bool{return a==b}
func notEqual106(a,b [106]byte)bool{return a!=b}
func equal107(a,b [107]byte)bool{return a==b}
func notEqual107(a,b [107]byte)bool{return a!=b}
func equal108(a,b [108]byte)bool{return a==b}
func notEqual108(a,b [108]byte)bool{return a!=b}
func equal109(a,b [109]byte)bool{return a==b}
func notEqual109(a,b [109]byte)bool{return a!=b}
func equal110(a,b [110]byte)bool{return a==b}
func notEqual110(a,b [110]byte)bool{return a!=b}
func equal111(a,b [111]byte)bool{return a==b}
func notEqual111(a,b [111]byte)bool{return a!=b}
func equal112(a,b [112]byte)bool{return a==b}
func notEqual112(a,b [112]byte)bool{return a!=b}
func equal113(a,b [113]byte)bool{return a==b}
func notEqual113(a,b [113]byte)bool{return a!=b}
func equal114(a,b [114]byte)bool{return a==b}
func notEqual114(a,b [114]byte)bool{return a!=b}
func equal115(a,b [115]byte)bool{return a==b}
func notEqual115(a,b [115]byte)bool{return a!=b}
func equal116(a,b [116]byte)bool{return a==b}
func notEqual116(a,b [116]byte)bool{return a!=b}
func equal117(a,b [117]byte)bool{return a==b}
func notEqual117(a,b [117]byte)bool{return a!=b}
func equal118(a,b [118]byte)bool{return a==b}
func notEqual118(a,b [118]byte)bool{return a!=b}
func equal119(a,b [119]byte)bool{return a==b}
func notEqual119(a,b [119]byte)bool{return a!=b}
func equal120(a,b [120]byte)bool{return a==b}
func notEqual120(a,b [120]byte)bool{return a!=b}
func equal121(a,b [121]byte)bool{return a==b}
func notEqual121(a,b [121]byte)bool{return a!=b}
func equal122(a,b [122]byte)bool{return a==b}
func notEqual122(a,b [122]byte)bool{return a!=b}
func equal123(a,b [123]byte)bool{return a==b}
func notEqual123(a,b [123]byte)bool{return a!=b}
func equal124(a,b [124]byte)bool{return a==b}
func notEqual124(a,b [124]byte)bool{return a!=b}
func equal125(a,b [125]byte)bool{return a==b}
func notEqual125(a,b [125]byte)bool{return a!=b}
func equal126(a,b [126]byte)bool{return a==b}
func notEqual126(a,b [126]byte)bool{return a!=b}
func equal127(a,b [127]byte)bool{return a==b}
func notEqual127(a,b [127]byte)bool{return a!=b}
func equal128(a,b [128]byte)bool{return a==b}
func notEqual128(a,b [128]byte)bool{return a!=b}
func equal129(a,b [129]byte)bool{return a==b}
func notEqual129(a,b [129]byte)bool{return a!=b}
func equal130(a,b [130]byte)bool{return a==b}
func notEqual130(a,b [130]byte)bool{return a!=b}
func equal131(a,b [131]byte)bool{return a==b}
func notEqual131(a,b [131]byte)bool{return a!=b}
func equal132(a,b [132]byte)bool{return a==b}
func notEqual132(a,b [132]byte)bool{return a!=b}
func equal133(a,b [133]byte)bool{return a==b}
func notEqual133(a,b [133]byte)bool{return a!=b}
func equal134(a,b [134]byte)bool{return a==b}
func notEqual134(a,b [134]byte)bool{return a!=b}
func equal135(a,b [135]byte)bool{return a==b}
func notEqual135(a,b [135]byte)bool{return a!=b}
func equal136(a,b [136]byte)bool{return a==b}
func notEqual136(a,b [136]byte)bool{return a!=b}
func equal137(a,b [137]byte)bool{return a==b}
func notEqual137(a,b [137]byte)bool{return a!=b}
func equal138(a,b [138]byte)bool{return a==b}
func notEqual138(a,b [138]byte)bool{return a!=b}
func equal139(a,b [139]byte)bool{return a==b}
func notEqual139(a,b [139]byte)bool{return a!=b}
func equal140(a,b [140]byte)bool{return a==b}
func notEqual140(a,b [140]byte)bool{return a!=b}
func equal141(a,b [141]byte)bool{return a==b}
func notEqual141(a,b [141]byte)bool{return a!=b}
func equal142(a,b [142]byte)bool{return a==b}
func notEqual142(a,b [142]byte)bool{return a!=b}
func equal143(a,b [143]byte)bool{return a==b}
func notEqual143(a,b [143]byte)bool{return a!=b}
func equal144(a,b [144]byte)bool{return a==b}
func notEqual144(a,b [144]byte)bool{return a!=b}
func equal145(a,b [145]byte)bool{return a==b}
func notEqual145(a,b [145]byte)bool{return a!=b}
func equal146(a,b [146]byte)bool{return a==b}
func notEqual146(a,b [146]byte)bool{return a!=b}
func equal147(a,b [147]byte)bool{return a==b}
func notEqual147(a,b [147]byte)bool{return a!=b}
func equal148(a,b [148]byte)bool{return a==b}
func notEqual148(a,b [148]byte)bool{return a!=b}
func equal149(a,b [149]byte)bool{return a==b}
func notEqual149(a,b [149]byte)bool{return a!=b}
func equal150(a,b [150]byte)bool{return a==b}
func notEqual150(a,b [150]byte)bool{return a!=b}
func equal151(a,b [151]byte)bool{return a==b}
func notEqual151(a,b [151]byte)bool{return a!=b}
func equal152(a,b [152]byte)bool{return a==b}
func notEqual152(a,b [152]byte)bool{return a!=b}
func equal153(a,b [153]byte)bool{return a==b}
func notEqual153(a,b [153]byte)bool{return a!=b}
func equal154(a,b [154]byte)bool{return a==b}
func notEqual154(a,b [154]byte)bool{return a!=b}
func equal155(a,b [155]byte)bool{return a==b}
func notEqual155(a,b [155]byte)bool{return a!=b}
func equal156(a,b [156]byte)bool{return a==b}
func notEqual156(a,b [156]byte)bool{return a!=b}
func equal157(a,b [157]byte)bool{return a==b}
func notEqual157(a,b [157]byte)bool{return a!=b}
func equal158(a,b [158]byte)bool{return a==b}
func notEqual158(a,b [158]byte)bool{return a!=b}
func equal159(a,b [159]byte)bool{return a==b}
func notEqual159(a,b [159]byte)bool{return a!=b}
func equal160(a,b [160]byte)bool{return a==b}
func notEqual160(a,b [160]byte)bool{return a!=b}
func equal161(a,b [161]byte)bool{return a==b}
func notEqual161(a,b [161]byte)bool{return a!=b}
func equal162(a,b [162]byte)bool{return a==b}
func notEqual162(a,b [162]byte)bool{return a!=b}
func equal163(a,b [163]byte)bool{return a==b}
func notEqual163(a,b [163]byte)bool{return a!=b}
func equal164(a,b [164]byte)bool{return a==b}
func notEqual164(a,b [164]byte)bool{return a!=b}
func equal165(a,b [165]byte)bool{return a==b}
func notEqual165(a,b [165]byte)bool{return a!=b}
func equal166(a,b [166]byte)bool{return a==b}
func notEqual166(a,b [166]byte)bool{return a!=b}
func equal167(a,b [167]byte)bool{return a==b}
func notEqual167(a,b [167]byte)bool{return a!=b}
func equal168(a,b [168]byte)bool{return a==b}
func notEqual168(a,b [168]byte)bool{return a!=b}
func equal169(a,b [169]byte)bool{return a==b}
func notEqual169(a,b [169]byte)bool{return a!=b}
func equal170(a,b [170]byte)bool{return a==b}
func notEqual170(a,b [170]byte)bool{return a!=b}
func equal171(a,b [171]byte)bool{return a==b}
func notEqual171(a,b [171]byte)bool{return a!=b}
func equal172(a,b [172]byte)bool{return a==b}
func notEqual172(a,b [172]byte)bool{return a!=b}
func equal173(a,b [173]byte)bool{return a==b}
func notEqual173(a,b [173]byte)bool{return a!=b}
func equal174(a,b [174]byte)bool{return a==b}
func notEqual174(a,b [174]byte)bool{return a!=b}
func equal175(a,b [175]byte)bool{return a==b}
func notEqual175(a,b [175]byte)bool{return a!=b}
func equal176(a,b [176]byte)bool{return a==b}
func notEqual176(a,b [176]byte)bool{return a!=b}
func equal177(a,b [177]byte)bool{return a==b}
func notEqual177(a,b [177]byte)bool{return a!=b}
func equal178(a,b [178]byte)bool{return a==b}
func notEqual178(a,b [178]byte)bool{return a!=b}
func equal179(a,b [179]byte)bool{return a==b}
func notEqual179(a,b [179]byte)bool{return a!=b}
func equal180(a,b [180]byte)bool{return a==b}
func notEqual180(a,b [180]byte)bool{return a!=b}
func equal181(a,b [181]byte)bool{return a==b}
func notEqual181(a,b [181]byte)bool{return a!=b}
func equal182(a,b [182]byte)bool{return a==b}
func notEqual182(a,b [182]byte)bool{return a!=b}
func equal183(a,b [183]byte)bool{return a==b}
func notEqual183(a,b [183]byte)bool{return a!=b}
func equal184(a,b [184]byte)bool{return a==b}
func notEqual184(a,b [184]byte)bool{return a!=b}
func equal185(a,b [185]byte)bool{return a==b}
func notEqual185(a,b [185]byte)bool{return a!=b}
func equal186(a,b [186]byte)bool{return a==b}
func notEqual186(a,b [186]byte)bool{return a!=b}
func equal187(a,b [187]byte)bool{return a==b}
func notEqual187(a,b [187]byte)bool{return a!=b}
func equal188(a,b [188]byte)bool{return a==b}
func notEqual188(a,b [188]byte)bool{return a!=b}
func equal189(a,b [189]byte)bool{return a==b}
func notEqual189(a,b [189]byte)bool{return a!=b}
func equal190(a,b [190]byte)bool{return a==b}
func notEqual190(a,b [190]byte)bool{return a!=b}
func equal191(a,b [191]byte)bool{return a==b}
func notEqual191(a,b [191]byte)bool{return a!=b}
func equal192(a,b [192]byte)bool{return a==b}
func notEqual192(a,b [192]byte)bool{return a!=b}
func equal193(a,b [193]byte)bool{return a==b}
func notEqual193(a,b [193]byte)bool{return a!=b}
func equal194(a,b [194]byte)bool{return a==b}
func notEqual194(a,b [194]byte)bool{return a!=b}
func equal195(a,b [195]byte)bool{return a==b}
func notEqual195(a,b [195]byte)bool{return a!=b}
func equal196(a,b [196]byte)bool{return a==b}
func notEqual196(a,b [196]byte)bool{return a!=b}
func equal197(a,b [197]byte)bool{return a==b}
func notEqual197(a,b [197]byte)bool{return a!=b}
func equal198(a,b [198]byte)bool{return a==b}
func notEqual198(a,b [198]byte)bool{return a!=b}
func equal199(a,b [199]byte)bool{return a==b}
func notEqual199(a,b [199]byte)bool{return a!=b}
func equal200(a,b [200]byte)bool{return a==b}
func notEqual200(a,b [200]byte)bool{return a!=b}
func equal201(a,b [201]byte)bool{return a==b}
func notEqual201(a,b [201]byte)bool{return a!=b}
func equal202(a,b [202]byte)bool{return a==b}
func notEqual202(a,b [202]byte)bool{return a!=b}
func equal203(a,b [203]byte)bool{return a==b}
func notEqual203(a,b [203]byte)bool{return a!=b}
func equal204(a,b [204]byte)bool{return a==b}
func notEqual204(a,b [204]byte)bool{return a!=b}
func equal205(a,b [205]byte)bool{return a==b}
func notEqual205(a,b [205]byte)bool{return a!=b}
func equal206(a,b [206]byte)bool{return a==b}
func notEqual206(a,b [206]byte)bool{return a!=b}
func equal207(a,b [207]byte)bool{return a==b}
func notEqual207(a,b [207]byte)bool{return a!=b}
func equal208(a,b [208]byte)bool{return a==b}
func notEqual208(a,b [208]byte)bool{return a!=b}
func equal209(a,b [209]byte)bool{return a==b}
func notEqual209(a,b [209]byte)bool{return a!=b}
func equal210(a,b [210]byte)bool{return a==b}
func notEqual210(a,b [210]byte)bool{return a!=b}
func equal211(a,b [211]byte)bool{return a==b}
func notEqual211(a,b [211]byte)bool{return a!=b}
func equal212(a,b [212]byte)bool{return a==b}
func notEqual212(a,b [212]byte)bool{return a!=b}
func equal213(a,b [213]byte)bool{return a==b}
func notEqual213(a,b [213]byte)bool{return a!=b}
func equal214(a,b [214]byte)bool{return a==b}
func notEqual214(a,b [214]byte)bool{return a!=b}
func equal215(a,b [215]byte)bool{return a==b}
func notEqual215(a,b [215]byte)bool{return a!=b}
func equal216(a,b [216]byte)bool{return a==b}
func notEqual216(a,b [216]byte)bool{return a!=b}
func equal217(a,b [217]byte)bool{return a==b}
func notEqual217(a,b [217]byte)bool{return a!=b}
func equal218(a,b [218]byte)bool{return a==b}
func notEqual218(a,b [218]byte)bool{return a!=b}
func equal219(a,b [219]byte)bool{return a==b}
func notEqual219(a,b [219]byte)bool{return a!=b}
func equal220(a,b [220]byte)bool{return a==b}
func notEqual220(a,b [220]byte)bool{return a!=b}
func equal221(a,b [221]byte)bool{return a==b}
func notEqual221(a,b [221]byte)bool{return a!=b}
func equal222(a,b [222]byte)bool{return a==b}
func notEqual222(a,b [222]byte)bool{return a!=b}
func equal223(a,b [223]byte)bool{return a==b}
func notEqual223(a,b [223]byte)bool{return a!=b}
func equal224(a,b [224]byte)bool{return a==b}
func notEqual224(a,b [224]byte)bool{return a!=b}
func equal225(a,b [225]byte)bool{return a==b}
func notEqual225(a,b [225]byte)bool{return a!=b}
func equal226(a,b [226]byte)bool{return a==b}
func notEqual226(a,b [226]byte)bool{return a!=b}
func equal227(a,b [227]byte)bool{return a==b}
func notEqual227(a,b [227]byte)bool{return a!=b}
func equal228(a,b [228]byte)bool{return a==b}
func notEqual228(a,b [228]byte)bool{return a!=b}
func equal229(a,b [229]byte)bool{return a==b}
func notEqual229(a,b [229]byte)bool{return a!=b}
func equal230(a,b [230]byte)bool{return a==b}
func notEqual230(a,b [230]byte)bool{return a!=b}
func equal231(a,b [231]byte)bool{return a==b}
func notEqual231(a,b [231]byte)bool{return a!=b}
func equal232(a,b [232]byte)bool{return a==b}
func notEqual232(a,b [232]byte)bool{return a!=b}
func equal233(a,b [233]byte)bool{return a==b}
func notEqual233(a,b [233]byte)bool{return a!=b}
func equal234(a,b [234]byte)bool{return a==b}
func notEqual234(a,b [234]byte)bool{return a!=b}
func equal235(a,b [235]byte)bool{return a==b}
func notEqual235(a,b [235]byte)bool{return a!=b}
func equal236(a,b [236]byte)bool{return a==b}
func notEqual236(a,b [236]byte)bool{return a!=b}
func equal237(a,b [237]byte)bool{return a==b}
func notEqual237(a,b [237]byte)bool{return a!=b}
func equal238(a,b [238]byte)bool{return a==b}
func notEqual238(a,b [238]byte)bool{return a!=b}
func equal239(a,b [239]byte)bool{return a==b}
func notEqual239(a,b [239]byte)bool{return a!=b}
func equal240(a,b [240]byte)bool{return a==b}
func notEqual240(a,b [240]byte)bool{return a!=b}
func equal241(a,b [241]byte)bool{return a==b}
func notEqual241(a,b [241]byte)bool{return a!=b}
func equal242(a,b [242]byte)bool{return a==b}
func notEqual242(a,b [242]byte)bool{return a!=b}
func equal243(a,b [243]byte)bool{return a==b}
func notEqual243(a,b [243]byte)bool{return a!=b}
func equal244(a,b [244]byte)bool{return a==b}
func notEqual244(a,b [244]byte)bool{return a!=b}
func equal245(a,b [245]byte)bool{return a==b}
func notEqual245(a,b [245]byte)bool{return a!=b}
func equal246(a,b [246]byte)bool{return a==b}
func notEqual246(a,b [246]byte)bool{return a!=b}
func equal247(a,b [247]byte)bool{return a==b}
func notEqual247(a,b [247]byte)bool{return a!=b}
func equal248(a,b [248]byte)bool{return a==b}
func notEqual248(a,b [248]byte)bool{return a!=b}
func equal249(a,b [249]byte)bool{return a==b}
func notEqual249(a,b [249]byte)bool{return a!=b}
func equal250(a,b [250]byte)bool{return a==b}
func notEqual250(a,b [250]byte)bool{return a!=b}
func equal251(a,b [251]byte)bool{return a==b}
func notEqual251(a,b [251]byte)bool{return a!=b}
func equal252(a,b [252]byte)bool{return a==b}
func notEqual252(a,b [252]byte)bool{return a!=b}
func equal253(a,b [253]byte)bool{return a==b}
func notEqual253(a,b [253]byte)bool{return a!=b}
func equal254(a,b [254]byte)bool{return a==b}
func notEqual254(a,b [254]byte)bool{return a!=b}
func equal255(a,b [255]byte)bool{return a==b}
func notEqual255(a,b [255]byte)bool{return a!=b}
func equal256(a,b [256]byte)bool{return a==b}
func notEqual256(a,b [256]byte)bool{return a!=b}
func compare(in Input)Output{out:=Output{ID:in.ID,Length:in.Length};if len(in.Left)!=256||len(in.Right)!=256{panic("whole input bytes")};switch in.Length{
case 0:var a,b [0]byte;for i:=0;i<0;i++{a[i]=byte(in.Left[i]);b[i]=byte(in.Right[i])};out.Equal=equal0(a,b);out.NotEqual=notEqual0(a,b)
case 1:var a,b [1]byte;for i:=0;i<1;i++{a[i]=byte(in.Left[i]);b[i]=byte(in.Right[i])};out.Equal=equal1(a,b);out.NotEqual=notEqual1(a,b)
case 2:var a,b [2]byte;for i:=0;i<2;i++{a[i]=byte(in.Left[i]);b[i]=byte(in.Right[i])};out.Equal=equal2(a,b);out.NotEqual=notEqual2(a,b)
case 3:var a,b [3]byte;for i:=0;i<3;i++{a[i]=byte(in.Left[i]);b[i]=byte(in.Right[i])};out.Equal=equal3(a,b);out.NotEqual=notEqual3(a,b)
case 4:var a,b [4]byte;for i:=0;i<4;i++{a[i]=byte(in.Left[i]);b[i]=byte(in.Right[i])};out.Equal=equal4(a,b);out.NotEqual=notEqual4(a,b)
case 5:var a,b [5]byte;for i:=0;i<5;i++{a[i]=byte(in.Left[i]);b[i]=byte(in.Right[i])};out.Equal=equal5(a,b);out.NotEqual=notEqual5(a,b)
case 6:var a,b [6]byte;for i:=0;i<6;i++{a[i]=byte(in.Left[i]);b[i]=byte(in.Right[i])};out.Equal=equal6(a,b);out.NotEqual=notEqual6(a,b)
case 7:var a,b [7]byte;for i:=0;i<7;i++{a[i]=byte(in.Left[i]);b[i]=byte(in.Right[i])};out.Equal=equal7(a,b);out.NotEqual=notEqual7(a,b)
case 8:var a,b [8]byte;for i:=0;i<8;i++{a[i]=byte(in.Left[i]);b[i]=byte(in.Right[i])};out.Equal=equal8(a,b);out.NotEqual=notEqual8(a,b)
case 9:var a,b [9]byte;for i:=0;i<9;i++{a[i]=byte(in.Left[i]);b[i]=byte(in.Right[i])};out.Equal=equal9(a,b);out.NotEqual=notEqual9(a,b)
case 10:var a,b [10]byte;for i:=0;i<10;i++{a[i]=byte(in.Left[i]);b[i]=byte(in.Right[i])};out.Equal=equal10(a,b);out.NotEqual=notEqual10(a,b)
case 11:var a,b [11]byte;for i:=0;i<11;i++{a[i]=byte(in.Left[i]);b[i]=byte(in.Right[i])};out.Equal=equal11(a,b);out.NotEqual=notEqual11(a,b)
case 12:var a,b [12]byte;for i:=0;i<12;i++{a[i]=byte(in.Left[i]);b[i]=byte(in.Right[i])};out.Equal=equal12(a,b);out.NotEqual=notEqual12(a,b)
case 13:var a,b [13]byte;for i:=0;i<13;i++{a[i]=byte(in.Left[i]);b[i]=byte(in.Right[i])};out.Equal=equal13(a,b);out.NotEqual=notEqual13(a,b)
case 14:var a,b [14]byte;for i:=0;i<14;i++{a[i]=byte(in.Left[i]);b[i]=byte(in.Right[i])};out.Equal=equal14(a,b);out.NotEqual=notEqual14(a,b)
case 15:var a,b [15]byte;for i:=0;i<15;i++{a[i]=byte(in.Left[i]);b[i]=byte(in.Right[i])};out.Equal=equal15(a,b);out.NotEqual=notEqual15(a,b)
case 16:var a,b [16]byte;for i:=0;i<16;i++{a[i]=byte(in.Left[i]);b[i]=byte(in.Right[i])};out.Equal=equal16(a,b);out.NotEqual=notEqual16(a,b)
case 17:var a,b [17]byte;for i:=0;i<17;i++{a[i]=byte(in.Left[i]);b[i]=byte(in.Right[i])};out.Equal=equal17(a,b);out.NotEqual=notEqual17(a,b)
case 18:var a,b [18]byte;for i:=0;i<18;i++{a[i]=byte(in.Left[i]);b[i]=byte(in.Right[i])};out.Equal=equal18(a,b);out.NotEqual=notEqual18(a,b)
case 19:var a,b [19]byte;for i:=0;i<19;i++{a[i]=byte(in.Left[i]);b[i]=byte(in.Right[i])};out.Equal=equal19(a,b);out.NotEqual=notEqual19(a,b)
case 20:var a,b [20]byte;for i:=0;i<20;i++{a[i]=byte(in.Left[i]);b[i]=byte(in.Right[i])};out.Equal=equal20(a,b);out.NotEqual=notEqual20(a,b)
case 21:var a,b [21]byte;for i:=0;i<21;i++{a[i]=byte(in.Left[i]);b[i]=byte(in.Right[i])};out.Equal=equal21(a,b);out.NotEqual=notEqual21(a,b)
case 22:var a,b [22]byte;for i:=0;i<22;i++{a[i]=byte(in.Left[i]);b[i]=byte(in.Right[i])};out.Equal=equal22(a,b);out.NotEqual=notEqual22(a,b)
case 23:var a,b [23]byte;for i:=0;i<23;i++{a[i]=byte(in.Left[i]);b[i]=byte(in.Right[i])};out.Equal=equal23(a,b);out.NotEqual=notEqual23(a,b)
case 24:var a,b [24]byte;for i:=0;i<24;i++{a[i]=byte(in.Left[i]);b[i]=byte(in.Right[i])};out.Equal=equal24(a,b);out.NotEqual=notEqual24(a,b)
case 25:var a,b [25]byte;for i:=0;i<25;i++{a[i]=byte(in.Left[i]);b[i]=byte(in.Right[i])};out.Equal=equal25(a,b);out.NotEqual=notEqual25(a,b)
case 26:var a,b [26]byte;for i:=0;i<26;i++{a[i]=byte(in.Left[i]);b[i]=byte(in.Right[i])};out.Equal=equal26(a,b);out.NotEqual=notEqual26(a,b)
case 27:var a,b [27]byte;for i:=0;i<27;i++{a[i]=byte(in.Left[i]);b[i]=byte(in.Right[i])};out.Equal=equal27(a,b);out.NotEqual=notEqual27(a,b)
case 28:var a,b [28]byte;for i:=0;i<28;i++{a[i]=byte(in.Left[i]);b[i]=byte(in.Right[i])};out.Equal=equal28(a,b);out.NotEqual=notEqual28(a,b)
case 29:var a,b [29]byte;for i:=0;i<29;i++{a[i]=byte(in.Left[i]);b[i]=byte(in.Right[i])};out.Equal=equal29(a,b);out.NotEqual=notEqual29(a,b)
case 30:var a,b [30]byte;for i:=0;i<30;i++{a[i]=byte(in.Left[i]);b[i]=byte(in.Right[i])};out.Equal=equal30(a,b);out.NotEqual=notEqual30(a,b)
case 31:var a,b [31]byte;for i:=0;i<31;i++{a[i]=byte(in.Left[i]);b[i]=byte(in.Right[i])};out.Equal=equal31(a,b);out.NotEqual=notEqual31(a,b)
case 32:var a,b [32]byte;for i:=0;i<32;i++{a[i]=byte(in.Left[i]);b[i]=byte(in.Right[i])};out.Equal=equal32(a,b);out.NotEqual=notEqual32(a,b)
case 33:var a,b [33]byte;for i:=0;i<33;i++{a[i]=byte(in.Left[i]);b[i]=byte(in.Right[i])};out.Equal=equal33(a,b);out.NotEqual=notEqual33(a,b)
case 34:var a,b [34]byte;for i:=0;i<34;i++{a[i]=byte(in.Left[i]);b[i]=byte(in.Right[i])};out.Equal=equal34(a,b);out.NotEqual=notEqual34(a,b)
case 35:var a,b [35]byte;for i:=0;i<35;i++{a[i]=byte(in.Left[i]);b[i]=byte(in.Right[i])};out.Equal=equal35(a,b);out.NotEqual=notEqual35(a,b)
case 36:var a,b [36]byte;for i:=0;i<36;i++{a[i]=byte(in.Left[i]);b[i]=byte(in.Right[i])};out.Equal=equal36(a,b);out.NotEqual=notEqual36(a,b)
case 37:var a,b [37]byte;for i:=0;i<37;i++{a[i]=byte(in.Left[i]);b[i]=byte(in.Right[i])};out.Equal=equal37(a,b);out.NotEqual=notEqual37(a,b)
case 38:var a,b [38]byte;for i:=0;i<38;i++{a[i]=byte(in.Left[i]);b[i]=byte(in.Right[i])};out.Equal=equal38(a,b);out.NotEqual=notEqual38(a,b)
case 39:var a,b [39]byte;for i:=0;i<39;i++{a[i]=byte(in.Left[i]);b[i]=byte(in.Right[i])};out.Equal=equal39(a,b);out.NotEqual=notEqual39(a,b)
case 40:var a,b [40]byte;for i:=0;i<40;i++{a[i]=byte(in.Left[i]);b[i]=byte(in.Right[i])};out.Equal=equal40(a,b);out.NotEqual=notEqual40(a,b)
case 41:var a,b [41]byte;for i:=0;i<41;i++{a[i]=byte(in.Left[i]);b[i]=byte(in.Right[i])};out.Equal=equal41(a,b);out.NotEqual=notEqual41(a,b)
case 42:var a,b [42]byte;for i:=0;i<42;i++{a[i]=byte(in.Left[i]);b[i]=byte(in.Right[i])};out.Equal=equal42(a,b);out.NotEqual=notEqual42(a,b)
case 43:var a,b [43]byte;for i:=0;i<43;i++{a[i]=byte(in.Left[i]);b[i]=byte(in.Right[i])};out.Equal=equal43(a,b);out.NotEqual=notEqual43(a,b)
case 44:var a,b [44]byte;for i:=0;i<44;i++{a[i]=byte(in.Left[i]);b[i]=byte(in.Right[i])};out.Equal=equal44(a,b);out.NotEqual=notEqual44(a,b)
case 45:var a,b [45]byte;for i:=0;i<45;i++{a[i]=byte(in.Left[i]);b[i]=byte(in.Right[i])};out.Equal=equal45(a,b);out.NotEqual=notEqual45(a,b)
case 46:var a,b [46]byte;for i:=0;i<46;i++{a[i]=byte(in.Left[i]);b[i]=byte(in.Right[i])};out.Equal=equal46(a,b);out.NotEqual=notEqual46(a,b)
case 47:var a,b [47]byte;for i:=0;i<47;i++{a[i]=byte(in.Left[i]);b[i]=byte(in.Right[i])};out.Equal=equal47(a,b);out.NotEqual=notEqual47(a,b)
case 48:var a,b [48]byte;for i:=0;i<48;i++{a[i]=byte(in.Left[i]);b[i]=byte(in.Right[i])};out.Equal=equal48(a,b);out.NotEqual=notEqual48(a,b)
case 49:var a,b [49]byte;for i:=0;i<49;i++{a[i]=byte(in.Left[i]);b[i]=byte(in.Right[i])};out.Equal=equal49(a,b);out.NotEqual=notEqual49(a,b)
case 50:var a,b [50]byte;for i:=0;i<50;i++{a[i]=byte(in.Left[i]);b[i]=byte(in.Right[i])};out.Equal=equal50(a,b);out.NotEqual=notEqual50(a,b)
case 51:var a,b [51]byte;for i:=0;i<51;i++{a[i]=byte(in.Left[i]);b[i]=byte(in.Right[i])};out.Equal=equal51(a,b);out.NotEqual=notEqual51(a,b)
case 52:var a,b [52]byte;for i:=0;i<52;i++{a[i]=byte(in.Left[i]);b[i]=byte(in.Right[i])};out.Equal=equal52(a,b);out.NotEqual=notEqual52(a,b)
case 53:var a,b [53]byte;for i:=0;i<53;i++{a[i]=byte(in.Left[i]);b[i]=byte(in.Right[i])};out.Equal=equal53(a,b);out.NotEqual=notEqual53(a,b)
case 54:var a,b [54]byte;for i:=0;i<54;i++{a[i]=byte(in.Left[i]);b[i]=byte(in.Right[i])};out.Equal=equal54(a,b);out.NotEqual=notEqual54(a,b)
case 55:var a,b [55]byte;for i:=0;i<55;i++{a[i]=byte(in.Left[i]);b[i]=byte(in.Right[i])};out.Equal=equal55(a,b);out.NotEqual=notEqual55(a,b)
case 56:var a,b [56]byte;for i:=0;i<56;i++{a[i]=byte(in.Left[i]);b[i]=byte(in.Right[i])};out.Equal=equal56(a,b);out.NotEqual=notEqual56(a,b)
case 57:var a,b [57]byte;for i:=0;i<57;i++{a[i]=byte(in.Left[i]);b[i]=byte(in.Right[i])};out.Equal=equal57(a,b);out.NotEqual=notEqual57(a,b)
case 58:var a,b [58]byte;for i:=0;i<58;i++{a[i]=byte(in.Left[i]);b[i]=byte(in.Right[i])};out.Equal=equal58(a,b);out.NotEqual=notEqual58(a,b)
case 59:var a,b [59]byte;for i:=0;i<59;i++{a[i]=byte(in.Left[i]);b[i]=byte(in.Right[i])};out.Equal=equal59(a,b);out.NotEqual=notEqual59(a,b)
case 60:var a,b [60]byte;for i:=0;i<60;i++{a[i]=byte(in.Left[i]);b[i]=byte(in.Right[i])};out.Equal=equal60(a,b);out.NotEqual=notEqual60(a,b)
case 61:var a,b [61]byte;for i:=0;i<61;i++{a[i]=byte(in.Left[i]);b[i]=byte(in.Right[i])};out.Equal=equal61(a,b);out.NotEqual=notEqual61(a,b)
case 62:var a,b [62]byte;for i:=0;i<62;i++{a[i]=byte(in.Left[i]);b[i]=byte(in.Right[i])};out.Equal=equal62(a,b);out.NotEqual=notEqual62(a,b)
case 63:var a,b [63]byte;for i:=0;i<63;i++{a[i]=byte(in.Left[i]);b[i]=byte(in.Right[i])};out.Equal=equal63(a,b);out.NotEqual=notEqual63(a,b)
case 64:var a,b [64]byte;for i:=0;i<64;i++{a[i]=byte(in.Left[i]);b[i]=byte(in.Right[i])};out.Equal=equal64(a,b);out.NotEqual=notEqual64(a,b)
case 65:var a,b [65]byte;for i:=0;i<65;i++{a[i]=byte(in.Left[i]);b[i]=byte(in.Right[i])};out.Equal=equal65(a,b);out.NotEqual=notEqual65(a,b)
case 66:var a,b [66]byte;for i:=0;i<66;i++{a[i]=byte(in.Left[i]);b[i]=byte(in.Right[i])};out.Equal=equal66(a,b);out.NotEqual=notEqual66(a,b)
case 67:var a,b [67]byte;for i:=0;i<67;i++{a[i]=byte(in.Left[i]);b[i]=byte(in.Right[i])};out.Equal=equal67(a,b);out.NotEqual=notEqual67(a,b)
case 68:var a,b [68]byte;for i:=0;i<68;i++{a[i]=byte(in.Left[i]);b[i]=byte(in.Right[i])};out.Equal=equal68(a,b);out.NotEqual=notEqual68(a,b)
case 69:var a,b [69]byte;for i:=0;i<69;i++{a[i]=byte(in.Left[i]);b[i]=byte(in.Right[i])};out.Equal=equal69(a,b);out.NotEqual=notEqual69(a,b)
case 70:var a,b [70]byte;for i:=0;i<70;i++{a[i]=byte(in.Left[i]);b[i]=byte(in.Right[i])};out.Equal=equal70(a,b);out.NotEqual=notEqual70(a,b)
case 71:var a,b [71]byte;for i:=0;i<71;i++{a[i]=byte(in.Left[i]);b[i]=byte(in.Right[i])};out.Equal=equal71(a,b);out.NotEqual=notEqual71(a,b)
case 72:var a,b [72]byte;for i:=0;i<72;i++{a[i]=byte(in.Left[i]);b[i]=byte(in.Right[i])};out.Equal=equal72(a,b);out.NotEqual=notEqual72(a,b)
case 73:var a,b [73]byte;for i:=0;i<73;i++{a[i]=byte(in.Left[i]);b[i]=byte(in.Right[i])};out.Equal=equal73(a,b);out.NotEqual=notEqual73(a,b)
case 74:var a,b [74]byte;for i:=0;i<74;i++{a[i]=byte(in.Left[i]);b[i]=byte(in.Right[i])};out.Equal=equal74(a,b);out.NotEqual=notEqual74(a,b)
case 75:var a,b [75]byte;for i:=0;i<75;i++{a[i]=byte(in.Left[i]);b[i]=byte(in.Right[i])};out.Equal=equal75(a,b);out.NotEqual=notEqual75(a,b)
case 76:var a,b [76]byte;for i:=0;i<76;i++{a[i]=byte(in.Left[i]);b[i]=byte(in.Right[i])};out.Equal=equal76(a,b);out.NotEqual=notEqual76(a,b)
case 77:var a,b [77]byte;for i:=0;i<77;i++{a[i]=byte(in.Left[i]);b[i]=byte(in.Right[i])};out.Equal=equal77(a,b);out.NotEqual=notEqual77(a,b)
case 78:var a,b [78]byte;for i:=0;i<78;i++{a[i]=byte(in.Left[i]);b[i]=byte(in.Right[i])};out.Equal=equal78(a,b);out.NotEqual=notEqual78(a,b)
case 79:var a,b [79]byte;for i:=0;i<79;i++{a[i]=byte(in.Left[i]);b[i]=byte(in.Right[i])};out.Equal=equal79(a,b);out.NotEqual=notEqual79(a,b)
case 80:var a,b [80]byte;for i:=0;i<80;i++{a[i]=byte(in.Left[i]);b[i]=byte(in.Right[i])};out.Equal=equal80(a,b);out.NotEqual=notEqual80(a,b)
case 81:var a,b [81]byte;for i:=0;i<81;i++{a[i]=byte(in.Left[i]);b[i]=byte(in.Right[i])};out.Equal=equal81(a,b);out.NotEqual=notEqual81(a,b)
case 82:var a,b [82]byte;for i:=0;i<82;i++{a[i]=byte(in.Left[i]);b[i]=byte(in.Right[i])};out.Equal=equal82(a,b);out.NotEqual=notEqual82(a,b)
case 83:var a,b [83]byte;for i:=0;i<83;i++{a[i]=byte(in.Left[i]);b[i]=byte(in.Right[i])};out.Equal=equal83(a,b);out.NotEqual=notEqual83(a,b)
case 84:var a,b [84]byte;for i:=0;i<84;i++{a[i]=byte(in.Left[i]);b[i]=byte(in.Right[i])};out.Equal=equal84(a,b);out.NotEqual=notEqual84(a,b)
case 85:var a,b [85]byte;for i:=0;i<85;i++{a[i]=byte(in.Left[i]);b[i]=byte(in.Right[i])};out.Equal=equal85(a,b);out.NotEqual=notEqual85(a,b)
case 86:var a,b [86]byte;for i:=0;i<86;i++{a[i]=byte(in.Left[i]);b[i]=byte(in.Right[i])};out.Equal=equal86(a,b);out.NotEqual=notEqual86(a,b)
case 87:var a,b [87]byte;for i:=0;i<87;i++{a[i]=byte(in.Left[i]);b[i]=byte(in.Right[i])};out.Equal=equal87(a,b);out.NotEqual=notEqual87(a,b)
case 88:var a,b [88]byte;for i:=0;i<88;i++{a[i]=byte(in.Left[i]);b[i]=byte(in.Right[i])};out.Equal=equal88(a,b);out.NotEqual=notEqual88(a,b)
case 89:var a,b [89]byte;for i:=0;i<89;i++{a[i]=byte(in.Left[i]);b[i]=byte(in.Right[i])};out.Equal=equal89(a,b);out.NotEqual=notEqual89(a,b)
case 90:var a,b [90]byte;for i:=0;i<90;i++{a[i]=byte(in.Left[i]);b[i]=byte(in.Right[i])};out.Equal=equal90(a,b);out.NotEqual=notEqual90(a,b)
case 91:var a,b [91]byte;for i:=0;i<91;i++{a[i]=byte(in.Left[i]);b[i]=byte(in.Right[i])};out.Equal=equal91(a,b);out.NotEqual=notEqual91(a,b)
case 92:var a,b [92]byte;for i:=0;i<92;i++{a[i]=byte(in.Left[i]);b[i]=byte(in.Right[i])};out.Equal=equal92(a,b);out.NotEqual=notEqual92(a,b)
case 93:var a,b [93]byte;for i:=0;i<93;i++{a[i]=byte(in.Left[i]);b[i]=byte(in.Right[i])};out.Equal=equal93(a,b);out.NotEqual=notEqual93(a,b)
case 94:var a,b [94]byte;for i:=0;i<94;i++{a[i]=byte(in.Left[i]);b[i]=byte(in.Right[i])};out.Equal=equal94(a,b);out.NotEqual=notEqual94(a,b)
case 95:var a,b [95]byte;for i:=0;i<95;i++{a[i]=byte(in.Left[i]);b[i]=byte(in.Right[i])};out.Equal=equal95(a,b);out.NotEqual=notEqual95(a,b)
case 96:var a,b [96]byte;for i:=0;i<96;i++{a[i]=byte(in.Left[i]);b[i]=byte(in.Right[i])};out.Equal=equal96(a,b);out.NotEqual=notEqual96(a,b)
case 97:var a,b [97]byte;for i:=0;i<97;i++{a[i]=byte(in.Left[i]);b[i]=byte(in.Right[i])};out.Equal=equal97(a,b);out.NotEqual=notEqual97(a,b)
case 98:var a,b [98]byte;for i:=0;i<98;i++{a[i]=byte(in.Left[i]);b[i]=byte(in.Right[i])};out.Equal=equal98(a,b);out.NotEqual=notEqual98(a,b)
case 99:var a,b [99]byte;for i:=0;i<99;i++{a[i]=byte(in.Left[i]);b[i]=byte(in.Right[i])};out.Equal=equal99(a,b);out.NotEqual=notEqual99(a,b)
case 100:var a,b [100]byte;for i:=0;i<100;i++{a[i]=byte(in.Left[i]);b[i]=byte(in.Right[i])};out.Equal=equal100(a,b);out.NotEqual=notEqual100(a,b)
case 101:var a,b [101]byte;for i:=0;i<101;i++{a[i]=byte(in.Left[i]);b[i]=byte(in.Right[i])};out.Equal=equal101(a,b);out.NotEqual=notEqual101(a,b)
case 102:var a,b [102]byte;for i:=0;i<102;i++{a[i]=byte(in.Left[i]);b[i]=byte(in.Right[i])};out.Equal=equal102(a,b);out.NotEqual=notEqual102(a,b)
case 103:var a,b [103]byte;for i:=0;i<103;i++{a[i]=byte(in.Left[i]);b[i]=byte(in.Right[i])};out.Equal=equal103(a,b);out.NotEqual=notEqual103(a,b)
case 104:var a,b [104]byte;for i:=0;i<104;i++{a[i]=byte(in.Left[i]);b[i]=byte(in.Right[i])};out.Equal=equal104(a,b);out.NotEqual=notEqual104(a,b)
case 105:var a,b [105]byte;for i:=0;i<105;i++{a[i]=byte(in.Left[i]);b[i]=byte(in.Right[i])};out.Equal=equal105(a,b);out.NotEqual=notEqual105(a,b)
case 106:var a,b [106]byte;for i:=0;i<106;i++{a[i]=byte(in.Left[i]);b[i]=byte(in.Right[i])};out.Equal=equal106(a,b);out.NotEqual=notEqual106(a,b)
case 107:var a,b [107]byte;for i:=0;i<107;i++{a[i]=byte(in.Left[i]);b[i]=byte(in.Right[i])};out.Equal=equal107(a,b);out.NotEqual=notEqual107(a,b)
case 108:var a,b [108]byte;for i:=0;i<108;i++{a[i]=byte(in.Left[i]);b[i]=byte(in.Right[i])};out.Equal=equal108(a,b);out.NotEqual=notEqual108(a,b)
case 109:var a,b [109]byte;for i:=0;i<109;i++{a[i]=byte(in.Left[i]);b[i]=byte(in.Right[i])};out.Equal=equal109(a,b);out.NotEqual=notEqual109(a,b)
case 110:var a,b [110]byte;for i:=0;i<110;i++{a[i]=byte(in.Left[i]);b[i]=byte(in.Right[i])};out.Equal=equal110(a,b);out.NotEqual=notEqual110(a,b)
case 111:var a,b [111]byte;for i:=0;i<111;i++{a[i]=byte(in.Left[i]);b[i]=byte(in.Right[i])};out.Equal=equal111(a,b);out.NotEqual=notEqual111(a,b)
case 112:var a,b [112]byte;for i:=0;i<112;i++{a[i]=byte(in.Left[i]);b[i]=byte(in.Right[i])};out.Equal=equal112(a,b);out.NotEqual=notEqual112(a,b)
case 113:var a,b [113]byte;for i:=0;i<113;i++{a[i]=byte(in.Left[i]);b[i]=byte(in.Right[i])};out.Equal=equal113(a,b);out.NotEqual=notEqual113(a,b)
case 114:var a,b [114]byte;for i:=0;i<114;i++{a[i]=byte(in.Left[i]);b[i]=byte(in.Right[i])};out.Equal=equal114(a,b);out.NotEqual=notEqual114(a,b)
case 115:var a,b [115]byte;for i:=0;i<115;i++{a[i]=byte(in.Left[i]);b[i]=byte(in.Right[i])};out.Equal=equal115(a,b);out.NotEqual=notEqual115(a,b)
case 116:var a,b [116]byte;for i:=0;i<116;i++{a[i]=byte(in.Left[i]);b[i]=byte(in.Right[i])};out.Equal=equal116(a,b);out.NotEqual=notEqual116(a,b)
case 117:var a,b [117]byte;for i:=0;i<117;i++{a[i]=byte(in.Left[i]);b[i]=byte(in.Right[i])};out.Equal=equal117(a,b);out.NotEqual=notEqual117(a,b)
case 118:var a,b [118]byte;for i:=0;i<118;i++{a[i]=byte(in.Left[i]);b[i]=byte(in.Right[i])};out.Equal=equal118(a,b);out.NotEqual=notEqual118(a,b)
case 119:var a,b [119]byte;for i:=0;i<119;i++{a[i]=byte(in.Left[i]);b[i]=byte(in.Right[i])};out.Equal=equal119(a,b);out.NotEqual=notEqual119(a,b)
case 120:var a,b [120]byte;for i:=0;i<120;i++{a[i]=byte(in.Left[i]);b[i]=byte(in.Right[i])};out.Equal=equal120(a,b);out.NotEqual=notEqual120(a,b)
case 121:var a,b [121]byte;for i:=0;i<121;i++{a[i]=byte(in.Left[i]);b[i]=byte(in.Right[i])};out.Equal=equal121(a,b);out.NotEqual=notEqual121(a,b)
case 122:var a,b [122]byte;for i:=0;i<122;i++{a[i]=byte(in.Left[i]);b[i]=byte(in.Right[i])};out.Equal=equal122(a,b);out.NotEqual=notEqual122(a,b)
case 123:var a,b [123]byte;for i:=0;i<123;i++{a[i]=byte(in.Left[i]);b[i]=byte(in.Right[i])};out.Equal=equal123(a,b);out.NotEqual=notEqual123(a,b)
case 124:var a,b [124]byte;for i:=0;i<124;i++{a[i]=byte(in.Left[i]);b[i]=byte(in.Right[i])};out.Equal=equal124(a,b);out.NotEqual=notEqual124(a,b)
case 125:var a,b [125]byte;for i:=0;i<125;i++{a[i]=byte(in.Left[i]);b[i]=byte(in.Right[i])};out.Equal=equal125(a,b);out.NotEqual=notEqual125(a,b)
case 126:var a,b [126]byte;for i:=0;i<126;i++{a[i]=byte(in.Left[i]);b[i]=byte(in.Right[i])};out.Equal=equal126(a,b);out.NotEqual=notEqual126(a,b)
case 127:var a,b [127]byte;for i:=0;i<127;i++{a[i]=byte(in.Left[i]);b[i]=byte(in.Right[i])};out.Equal=equal127(a,b);out.NotEqual=notEqual127(a,b)
case 128:var a,b [128]byte;for i:=0;i<128;i++{a[i]=byte(in.Left[i]);b[i]=byte(in.Right[i])};out.Equal=equal128(a,b);out.NotEqual=notEqual128(a,b)
case 129:var a,b [129]byte;for i:=0;i<129;i++{a[i]=byte(in.Left[i]);b[i]=byte(in.Right[i])};out.Equal=equal129(a,b);out.NotEqual=notEqual129(a,b)
case 130:var a,b [130]byte;for i:=0;i<130;i++{a[i]=byte(in.Left[i]);b[i]=byte(in.Right[i])};out.Equal=equal130(a,b);out.NotEqual=notEqual130(a,b)
case 131:var a,b [131]byte;for i:=0;i<131;i++{a[i]=byte(in.Left[i]);b[i]=byte(in.Right[i])};out.Equal=equal131(a,b);out.NotEqual=notEqual131(a,b)
case 132:var a,b [132]byte;for i:=0;i<132;i++{a[i]=byte(in.Left[i]);b[i]=byte(in.Right[i])};out.Equal=equal132(a,b);out.NotEqual=notEqual132(a,b)
case 133:var a,b [133]byte;for i:=0;i<133;i++{a[i]=byte(in.Left[i]);b[i]=byte(in.Right[i])};out.Equal=equal133(a,b);out.NotEqual=notEqual133(a,b)
case 134:var a,b [134]byte;for i:=0;i<134;i++{a[i]=byte(in.Left[i]);b[i]=byte(in.Right[i])};out.Equal=equal134(a,b);out.NotEqual=notEqual134(a,b)
case 135:var a,b [135]byte;for i:=0;i<135;i++{a[i]=byte(in.Left[i]);b[i]=byte(in.Right[i])};out.Equal=equal135(a,b);out.NotEqual=notEqual135(a,b)
case 136:var a,b [136]byte;for i:=0;i<136;i++{a[i]=byte(in.Left[i]);b[i]=byte(in.Right[i])};out.Equal=equal136(a,b);out.NotEqual=notEqual136(a,b)
case 137:var a,b [137]byte;for i:=0;i<137;i++{a[i]=byte(in.Left[i]);b[i]=byte(in.Right[i])};out.Equal=equal137(a,b);out.NotEqual=notEqual137(a,b)
case 138:var a,b [138]byte;for i:=0;i<138;i++{a[i]=byte(in.Left[i]);b[i]=byte(in.Right[i])};out.Equal=equal138(a,b);out.NotEqual=notEqual138(a,b)
case 139:var a,b [139]byte;for i:=0;i<139;i++{a[i]=byte(in.Left[i]);b[i]=byte(in.Right[i])};out.Equal=equal139(a,b);out.NotEqual=notEqual139(a,b)
case 140:var a,b [140]byte;for i:=0;i<140;i++{a[i]=byte(in.Left[i]);b[i]=byte(in.Right[i])};out.Equal=equal140(a,b);out.NotEqual=notEqual140(a,b)
case 141:var a,b [141]byte;for i:=0;i<141;i++{a[i]=byte(in.Left[i]);b[i]=byte(in.Right[i])};out.Equal=equal141(a,b);out.NotEqual=notEqual141(a,b)
case 142:var a,b [142]byte;for i:=0;i<142;i++{a[i]=byte(in.Left[i]);b[i]=byte(in.Right[i])};out.Equal=equal142(a,b);out.NotEqual=notEqual142(a,b)
case 143:var a,b [143]byte;for i:=0;i<143;i++{a[i]=byte(in.Left[i]);b[i]=byte(in.Right[i])};out.Equal=equal143(a,b);out.NotEqual=notEqual143(a,b)
case 144:var a,b [144]byte;for i:=0;i<144;i++{a[i]=byte(in.Left[i]);b[i]=byte(in.Right[i])};out.Equal=equal144(a,b);out.NotEqual=notEqual144(a,b)
case 145:var a,b [145]byte;for i:=0;i<145;i++{a[i]=byte(in.Left[i]);b[i]=byte(in.Right[i])};out.Equal=equal145(a,b);out.NotEqual=notEqual145(a,b)
case 146:var a,b [146]byte;for i:=0;i<146;i++{a[i]=byte(in.Left[i]);b[i]=byte(in.Right[i])};out.Equal=equal146(a,b);out.NotEqual=notEqual146(a,b)
case 147:var a,b [147]byte;for i:=0;i<147;i++{a[i]=byte(in.Left[i]);b[i]=byte(in.Right[i])};out.Equal=equal147(a,b);out.NotEqual=notEqual147(a,b)
case 148:var a,b [148]byte;for i:=0;i<148;i++{a[i]=byte(in.Left[i]);b[i]=byte(in.Right[i])};out.Equal=equal148(a,b);out.NotEqual=notEqual148(a,b)
case 149:var a,b [149]byte;for i:=0;i<149;i++{a[i]=byte(in.Left[i]);b[i]=byte(in.Right[i])};out.Equal=equal149(a,b);out.NotEqual=notEqual149(a,b)
case 150:var a,b [150]byte;for i:=0;i<150;i++{a[i]=byte(in.Left[i]);b[i]=byte(in.Right[i])};out.Equal=equal150(a,b);out.NotEqual=notEqual150(a,b)
case 151:var a,b [151]byte;for i:=0;i<151;i++{a[i]=byte(in.Left[i]);b[i]=byte(in.Right[i])};out.Equal=equal151(a,b);out.NotEqual=notEqual151(a,b)
case 152:var a,b [152]byte;for i:=0;i<152;i++{a[i]=byte(in.Left[i]);b[i]=byte(in.Right[i])};out.Equal=equal152(a,b);out.NotEqual=notEqual152(a,b)
case 153:var a,b [153]byte;for i:=0;i<153;i++{a[i]=byte(in.Left[i]);b[i]=byte(in.Right[i])};out.Equal=equal153(a,b);out.NotEqual=notEqual153(a,b)
case 154:var a,b [154]byte;for i:=0;i<154;i++{a[i]=byte(in.Left[i]);b[i]=byte(in.Right[i])};out.Equal=equal154(a,b);out.NotEqual=notEqual154(a,b)
case 155:var a,b [155]byte;for i:=0;i<155;i++{a[i]=byte(in.Left[i]);b[i]=byte(in.Right[i])};out.Equal=equal155(a,b);out.NotEqual=notEqual155(a,b)
case 156:var a,b [156]byte;for i:=0;i<156;i++{a[i]=byte(in.Left[i]);b[i]=byte(in.Right[i])};out.Equal=equal156(a,b);out.NotEqual=notEqual156(a,b)
case 157:var a,b [157]byte;for i:=0;i<157;i++{a[i]=byte(in.Left[i]);b[i]=byte(in.Right[i])};out.Equal=equal157(a,b);out.NotEqual=notEqual157(a,b)
case 158:var a,b [158]byte;for i:=0;i<158;i++{a[i]=byte(in.Left[i]);b[i]=byte(in.Right[i])};out.Equal=equal158(a,b);out.NotEqual=notEqual158(a,b)
case 159:var a,b [159]byte;for i:=0;i<159;i++{a[i]=byte(in.Left[i]);b[i]=byte(in.Right[i])};out.Equal=equal159(a,b);out.NotEqual=notEqual159(a,b)
case 160:var a,b [160]byte;for i:=0;i<160;i++{a[i]=byte(in.Left[i]);b[i]=byte(in.Right[i])};out.Equal=equal160(a,b);out.NotEqual=notEqual160(a,b)
case 161:var a,b [161]byte;for i:=0;i<161;i++{a[i]=byte(in.Left[i]);b[i]=byte(in.Right[i])};out.Equal=equal161(a,b);out.NotEqual=notEqual161(a,b)
case 162:var a,b [162]byte;for i:=0;i<162;i++{a[i]=byte(in.Left[i]);b[i]=byte(in.Right[i])};out.Equal=equal162(a,b);out.NotEqual=notEqual162(a,b)
case 163:var a,b [163]byte;for i:=0;i<163;i++{a[i]=byte(in.Left[i]);b[i]=byte(in.Right[i])};out.Equal=equal163(a,b);out.NotEqual=notEqual163(a,b)
case 164:var a,b [164]byte;for i:=0;i<164;i++{a[i]=byte(in.Left[i]);b[i]=byte(in.Right[i])};out.Equal=equal164(a,b);out.NotEqual=notEqual164(a,b)
case 165:var a,b [165]byte;for i:=0;i<165;i++{a[i]=byte(in.Left[i]);b[i]=byte(in.Right[i])};out.Equal=equal165(a,b);out.NotEqual=notEqual165(a,b)
case 166:var a,b [166]byte;for i:=0;i<166;i++{a[i]=byte(in.Left[i]);b[i]=byte(in.Right[i])};out.Equal=equal166(a,b);out.NotEqual=notEqual166(a,b)
case 167:var a,b [167]byte;for i:=0;i<167;i++{a[i]=byte(in.Left[i]);b[i]=byte(in.Right[i])};out.Equal=equal167(a,b);out.NotEqual=notEqual167(a,b)
case 168:var a,b [168]byte;for i:=0;i<168;i++{a[i]=byte(in.Left[i]);b[i]=byte(in.Right[i])};out.Equal=equal168(a,b);out.NotEqual=notEqual168(a,b)
case 169:var a,b [169]byte;for i:=0;i<169;i++{a[i]=byte(in.Left[i]);b[i]=byte(in.Right[i])};out.Equal=equal169(a,b);out.NotEqual=notEqual169(a,b)
case 170:var a,b [170]byte;for i:=0;i<170;i++{a[i]=byte(in.Left[i]);b[i]=byte(in.Right[i])};out.Equal=equal170(a,b);out.NotEqual=notEqual170(a,b)
case 171:var a,b [171]byte;for i:=0;i<171;i++{a[i]=byte(in.Left[i]);b[i]=byte(in.Right[i])};out.Equal=equal171(a,b);out.NotEqual=notEqual171(a,b)
case 172:var a,b [172]byte;for i:=0;i<172;i++{a[i]=byte(in.Left[i]);b[i]=byte(in.Right[i])};out.Equal=equal172(a,b);out.NotEqual=notEqual172(a,b)
case 173:var a,b [173]byte;for i:=0;i<173;i++{a[i]=byte(in.Left[i]);b[i]=byte(in.Right[i])};out.Equal=equal173(a,b);out.NotEqual=notEqual173(a,b)
case 174:var a,b [174]byte;for i:=0;i<174;i++{a[i]=byte(in.Left[i]);b[i]=byte(in.Right[i])};out.Equal=equal174(a,b);out.NotEqual=notEqual174(a,b)
case 175:var a,b [175]byte;for i:=0;i<175;i++{a[i]=byte(in.Left[i]);b[i]=byte(in.Right[i])};out.Equal=equal175(a,b);out.NotEqual=notEqual175(a,b)
case 176:var a,b [176]byte;for i:=0;i<176;i++{a[i]=byte(in.Left[i]);b[i]=byte(in.Right[i])};out.Equal=equal176(a,b);out.NotEqual=notEqual176(a,b)
case 177:var a,b [177]byte;for i:=0;i<177;i++{a[i]=byte(in.Left[i]);b[i]=byte(in.Right[i])};out.Equal=equal177(a,b);out.NotEqual=notEqual177(a,b)
case 178:var a,b [178]byte;for i:=0;i<178;i++{a[i]=byte(in.Left[i]);b[i]=byte(in.Right[i])};out.Equal=equal178(a,b);out.NotEqual=notEqual178(a,b)
case 179:var a,b [179]byte;for i:=0;i<179;i++{a[i]=byte(in.Left[i]);b[i]=byte(in.Right[i])};out.Equal=equal179(a,b);out.NotEqual=notEqual179(a,b)
case 180:var a,b [180]byte;for i:=0;i<180;i++{a[i]=byte(in.Left[i]);b[i]=byte(in.Right[i])};out.Equal=equal180(a,b);out.NotEqual=notEqual180(a,b)
case 181:var a,b [181]byte;for i:=0;i<181;i++{a[i]=byte(in.Left[i]);b[i]=byte(in.Right[i])};out.Equal=equal181(a,b);out.NotEqual=notEqual181(a,b)
case 182:var a,b [182]byte;for i:=0;i<182;i++{a[i]=byte(in.Left[i]);b[i]=byte(in.Right[i])};out.Equal=equal182(a,b);out.NotEqual=notEqual182(a,b)
case 183:var a,b [183]byte;for i:=0;i<183;i++{a[i]=byte(in.Left[i]);b[i]=byte(in.Right[i])};out.Equal=equal183(a,b);out.NotEqual=notEqual183(a,b)
case 184:var a,b [184]byte;for i:=0;i<184;i++{a[i]=byte(in.Left[i]);b[i]=byte(in.Right[i])};out.Equal=equal184(a,b);out.NotEqual=notEqual184(a,b)
case 185:var a,b [185]byte;for i:=0;i<185;i++{a[i]=byte(in.Left[i]);b[i]=byte(in.Right[i])};out.Equal=equal185(a,b);out.NotEqual=notEqual185(a,b)
case 186:var a,b [186]byte;for i:=0;i<186;i++{a[i]=byte(in.Left[i]);b[i]=byte(in.Right[i])};out.Equal=equal186(a,b);out.NotEqual=notEqual186(a,b)
case 187:var a,b [187]byte;for i:=0;i<187;i++{a[i]=byte(in.Left[i]);b[i]=byte(in.Right[i])};out.Equal=equal187(a,b);out.NotEqual=notEqual187(a,b)
case 188:var a,b [188]byte;for i:=0;i<188;i++{a[i]=byte(in.Left[i]);b[i]=byte(in.Right[i])};out.Equal=equal188(a,b);out.NotEqual=notEqual188(a,b)
case 189:var a,b [189]byte;for i:=0;i<189;i++{a[i]=byte(in.Left[i]);b[i]=byte(in.Right[i])};out.Equal=equal189(a,b);out.NotEqual=notEqual189(a,b)
case 190:var a,b [190]byte;for i:=0;i<190;i++{a[i]=byte(in.Left[i]);b[i]=byte(in.Right[i])};out.Equal=equal190(a,b);out.NotEqual=notEqual190(a,b)
case 191:var a,b [191]byte;for i:=0;i<191;i++{a[i]=byte(in.Left[i]);b[i]=byte(in.Right[i])};out.Equal=equal191(a,b);out.NotEqual=notEqual191(a,b)
case 192:var a,b [192]byte;for i:=0;i<192;i++{a[i]=byte(in.Left[i]);b[i]=byte(in.Right[i])};out.Equal=equal192(a,b);out.NotEqual=notEqual192(a,b)
case 193:var a,b [193]byte;for i:=0;i<193;i++{a[i]=byte(in.Left[i]);b[i]=byte(in.Right[i])};out.Equal=equal193(a,b);out.NotEqual=notEqual193(a,b)
case 194:var a,b [194]byte;for i:=0;i<194;i++{a[i]=byte(in.Left[i]);b[i]=byte(in.Right[i])};out.Equal=equal194(a,b);out.NotEqual=notEqual194(a,b)
case 195:var a,b [195]byte;for i:=0;i<195;i++{a[i]=byte(in.Left[i]);b[i]=byte(in.Right[i])};out.Equal=equal195(a,b);out.NotEqual=notEqual195(a,b)
case 196:var a,b [196]byte;for i:=0;i<196;i++{a[i]=byte(in.Left[i]);b[i]=byte(in.Right[i])};out.Equal=equal196(a,b);out.NotEqual=notEqual196(a,b)
case 197:var a,b [197]byte;for i:=0;i<197;i++{a[i]=byte(in.Left[i]);b[i]=byte(in.Right[i])};out.Equal=equal197(a,b);out.NotEqual=notEqual197(a,b)
case 198:var a,b [198]byte;for i:=0;i<198;i++{a[i]=byte(in.Left[i]);b[i]=byte(in.Right[i])};out.Equal=equal198(a,b);out.NotEqual=notEqual198(a,b)
case 199:var a,b [199]byte;for i:=0;i<199;i++{a[i]=byte(in.Left[i]);b[i]=byte(in.Right[i])};out.Equal=equal199(a,b);out.NotEqual=notEqual199(a,b)
case 200:var a,b [200]byte;for i:=0;i<200;i++{a[i]=byte(in.Left[i]);b[i]=byte(in.Right[i])};out.Equal=equal200(a,b);out.NotEqual=notEqual200(a,b)
case 201:var a,b [201]byte;for i:=0;i<201;i++{a[i]=byte(in.Left[i]);b[i]=byte(in.Right[i])};out.Equal=equal201(a,b);out.NotEqual=notEqual201(a,b)
case 202:var a,b [202]byte;for i:=0;i<202;i++{a[i]=byte(in.Left[i]);b[i]=byte(in.Right[i])};out.Equal=equal202(a,b);out.NotEqual=notEqual202(a,b)
case 203:var a,b [203]byte;for i:=0;i<203;i++{a[i]=byte(in.Left[i]);b[i]=byte(in.Right[i])};out.Equal=equal203(a,b);out.NotEqual=notEqual203(a,b)
case 204:var a,b [204]byte;for i:=0;i<204;i++{a[i]=byte(in.Left[i]);b[i]=byte(in.Right[i])};out.Equal=equal204(a,b);out.NotEqual=notEqual204(a,b)
case 205:var a,b [205]byte;for i:=0;i<205;i++{a[i]=byte(in.Left[i]);b[i]=byte(in.Right[i])};out.Equal=equal205(a,b);out.NotEqual=notEqual205(a,b)
case 206:var a,b [206]byte;for i:=0;i<206;i++{a[i]=byte(in.Left[i]);b[i]=byte(in.Right[i])};out.Equal=equal206(a,b);out.NotEqual=notEqual206(a,b)
case 207:var a,b [207]byte;for i:=0;i<207;i++{a[i]=byte(in.Left[i]);b[i]=byte(in.Right[i])};out.Equal=equal207(a,b);out.NotEqual=notEqual207(a,b)
case 208:var a,b [208]byte;for i:=0;i<208;i++{a[i]=byte(in.Left[i]);b[i]=byte(in.Right[i])};out.Equal=equal208(a,b);out.NotEqual=notEqual208(a,b)
case 209:var a,b [209]byte;for i:=0;i<209;i++{a[i]=byte(in.Left[i]);b[i]=byte(in.Right[i])};out.Equal=equal209(a,b);out.NotEqual=notEqual209(a,b)
case 210:var a,b [210]byte;for i:=0;i<210;i++{a[i]=byte(in.Left[i]);b[i]=byte(in.Right[i])};out.Equal=equal210(a,b);out.NotEqual=notEqual210(a,b)
case 211:var a,b [211]byte;for i:=0;i<211;i++{a[i]=byte(in.Left[i]);b[i]=byte(in.Right[i])};out.Equal=equal211(a,b);out.NotEqual=notEqual211(a,b)
case 212:var a,b [212]byte;for i:=0;i<212;i++{a[i]=byte(in.Left[i]);b[i]=byte(in.Right[i])};out.Equal=equal212(a,b);out.NotEqual=notEqual212(a,b)
case 213:var a,b [213]byte;for i:=0;i<213;i++{a[i]=byte(in.Left[i]);b[i]=byte(in.Right[i])};out.Equal=equal213(a,b);out.NotEqual=notEqual213(a,b)
case 214:var a,b [214]byte;for i:=0;i<214;i++{a[i]=byte(in.Left[i]);b[i]=byte(in.Right[i])};out.Equal=equal214(a,b);out.NotEqual=notEqual214(a,b)
case 215:var a,b [215]byte;for i:=0;i<215;i++{a[i]=byte(in.Left[i]);b[i]=byte(in.Right[i])};out.Equal=equal215(a,b);out.NotEqual=notEqual215(a,b)
case 216:var a,b [216]byte;for i:=0;i<216;i++{a[i]=byte(in.Left[i]);b[i]=byte(in.Right[i])};out.Equal=equal216(a,b);out.NotEqual=notEqual216(a,b)
case 217:var a,b [217]byte;for i:=0;i<217;i++{a[i]=byte(in.Left[i]);b[i]=byte(in.Right[i])};out.Equal=equal217(a,b);out.NotEqual=notEqual217(a,b)
case 218:var a,b [218]byte;for i:=0;i<218;i++{a[i]=byte(in.Left[i]);b[i]=byte(in.Right[i])};out.Equal=equal218(a,b);out.NotEqual=notEqual218(a,b)
case 219:var a,b [219]byte;for i:=0;i<219;i++{a[i]=byte(in.Left[i]);b[i]=byte(in.Right[i])};out.Equal=equal219(a,b);out.NotEqual=notEqual219(a,b)
case 220:var a,b [220]byte;for i:=0;i<220;i++{a[i]=byte(in.Left[i]);b[i]=byte(in.Right[i])};out.Equal=equal220(a,b);out.NotEqual=notEqual220(a,b)
case 221:var a,b [221]byte;for i:=0;i<221;i++{a[i]=byte(in.Left[i]);b[i]=byte(in.Right[i])};out.Equal=equal221(a,b);out.NotEqual=notEqual221(a,b)
case 222:var a,b [222]byte;for i:=0;i<222;i++{a[i]=byte(in.Left[i]);b[i]=byte(in.Right[i])};out.Equal=equal222(a,b);out.NotEqual=notEqual222(a,b)
case 223:var a,b [223]byte;for i:=0;i<223;i++{a[i]=byte(in.Left[i]);b[i]=byte(in.Right[i])};out.Equal=equal223(a,b);out.NotEqual=notEqual223(a,b)
case 224:var a,b [224]byte;for i:=0;i<224;i++{a[i]=byte(in.Left[i]);b[i]=byte(in.Right[i])};out.Equal=equal224(a,b);out.NotEqual=notEqual224(a,b)
case 225:var a,b [225]byte;for i:=0;i<225;i++{a[i]=byte(in.Left[i]);b[i]=byte(in.Right[i])};out.Equal=equal225(a,b);out.NotEqual=notEqual225(a,b)
case 226:var a,b [226]byte;for i:=0;i<226;i++{a[i]=byte(in.Left[i]);b[i]=byte(in.Right[i])};out.Equal=equal226(a,b);out.NotEqual=notEqual226(a,b)
case 227:var a,b [227]byte;for i:=0;i<227;i++{a[i]=byte(in.Left[i]);b[i]=byte(in.Right[i])};out.Equal=equal227(a,b);out.NotEqual=notEqual227(a,b)
case 228:var a,b [228]byte;for i:=0;i<228;i++{a[i]=byte(in.Left[i]);b[i]=byte(in.Right[i])};out.Equal=equal228(a,b);out.NotEqual=notEqual228(a,b)
case 229:var a,b [229]byte;for i:=0;i<229;i++{a[i]=byte(in.Left[i]);b[i]=byte(in.Right[i])};out.Equal=equal229(a,b);out.NotEqual=notEqual229(a,b)
case 230:var a,b [230]byte;for i:=0;i<230;i++{a[i]=byte(in.Left[i]);b[i]=byte(in.Right[i])};out.Equal=equal230(a,b);out.NotEqual=notEqual230(a,b)
case 231:var a,b [231]byte;for i:=0;i<231;i++{a[i]=byte(in.Left[i]);b[i]=byte(in.Right[i])};out.Equal=equal231(a,b);out.NotEqual=notEqual231(a,b)
case 232:var a,b [232]byte;for i:=0;i<232;i++{a[i]=byte(in.Left[i]);b[i]=byte(in.Right[i])};out.Equal=equal232(a,b);out.NotEqual=notEqual232(a,b)
case 233:var a,b [233]byte;for i:=0;i<233;i++{a[i]=byte(in.Left[i]);b[i]=byte(in.Right[i])};out.Equal=equal233(a,b);out.NotEqual=notEqual233(a,b)
case 234:var a,b [234]byte;for i:=0;i<234;i++{a[i]=byte(in.Left[i]);b[i]=byte(in.Right[i])};out.Equal=equal234(a,b);out.NotEqual=notEqual234(a,b)
case 235:var a,b [235]byte;for i:=0;i<235;i++{a[i]=byte(in.Left[i]);b[i]=byte(in.Right[i])};out.Equal=equal235(a,b);out.NotEqual=notEqual235(a,b)
case 236:var a,b [236]byte;for i:=0;i<236;i++{a[i]=byte(in.Left[i]);b[i]=byte(in.Right[i])};out.Equal=equal236(a,b);out.NotEqual=notEqual236(a,b)
case 237:var a,b [237]byte;for i:=0;i<237;i++{a[i]=byte(in.Left[i]);b[i]=byte(in.Right[i])};out.Equal=equal237(a,b);out.NotEqual=notEqual237(a,b)
case 238:var a,b [238]byte;for i:=0;i<238;i++{a[i]=byte(in.Left[i]);b[i]=byte(in.Right[i])};out.Equal=equal238(a,b);out.NotEqual=notEqual238(a,b)
case 239:var a,b [239]byte;for i:=0;i<239;i++{a[i]=byte(in.Left[i]);b[i]=byte(in.Right[i])};out.Equal=equal239(a,b);out.NotEqual=notEqual239(a,b)
case 240:var a,b [240]byte;for i:=0;i<240;i++{a[i]=byte(in.Left[i]);b[i]=byte(in.Right[i])};out.Equal=equal240(a,b);out.NotEqual=notEqual240(a,b)
case 241:var a,b [241]byte;for i:=0;i<241;i++{a[i]=byte(in.Left[i]);b[i]=byte(in.Right[i])};out.Equal=equal241(a,b);out.NotEqual=notEqual241(a,b)
case 242:var a,b [242]byte;for i:=0;i<242;i++{a[i]=byte(in.Left[i]);b[i]=byte(in.Right[i])};out.Equal=equal242(a,b);out.NotEqual=notEqual242(a,b)
case 243:var a,b [243]byte;for i:=0;i<243;i++{a[i]=byte(in.Left[i]);b[i]=byte(in.Right[i])};out.Equal=equal243(a,b);out.NotEqual=notEqual243(a,b)
case 244:var a,b [244]byte;for i:=0;i<244;i++{a[i]=byte(in.Left[i]);b[i]=byte(in.Right[i])};out.Equal=equal244(a,b);out.NotEqual=notEqual244(a,b)
case 245:var a,b [245]byte;for i:=0;i<245;i++{a[i]=byte(in.Left[i]);b[i]=byte(in.Right[i])};out.Equal=equal245(a,b);out.NotEqual=notEqual245(a,b)
case 246:var a,b [246]byte;for i:=0;i<246;i++{a[i]=byte(in.Left[i]);b[i]=byte(in.Right[i])};out.Equal=equal246(a,b);out.NotEqual=notEqual246(a,b)
case 247:var a,b [247]byte;for i:=0;i<247;i++{a[i]=byte(in.Left[i]);b[i]=byte(in.Right[i])};out.Equal=equal247(a,b);out.NotEqual=notEqual247(a,b)
case 248:var a,b [248]byte;for i:=0;i<248;i++{a[i]=byte(in.Left[i]);b[i]=byte(in.Right[i])};out.Equal=equal248(a,b);out.NotEqual=notEqual248(a,b)
case 249:var a,b [249]byte;for i:=0;i<249;i++{a[i]=byte(in.Left[i]);b[i]=byte(in.Right[i])};out.Equal=equal249(a,b);out.NotEqual=notEqual249(a,b)
case 250:var a,b [250]byte;for i:=0;i<250;i++{a[i]=byte(in.Left[i]);b[i]=byte(in.Right[i])};out.Equal=equal250(a,b);out.NotEqual=notEqual250(a,b)
case 251:var a,b [251]byte;for i:=0;i<251;i++{a[i]=byte(in.Left[i]);b[i]=byte(in.Right[i])};out.Equal=equal251(a,b);out.NotEqual=notEqual251(a,b)
case 252:var a,b [252]byte;for i:=0;i<252;i++{a[i]=byte(in.Left[i]);b[i]=byte(in.Right[i])};out.Equal=equal252(a,b);out.NotEqual=notEqual252(a,b)
case 253:var a,b [253]byte;for i:=0;i<253;i++{a[i]=byte(in.Left[i]);b[i]=byte(in.Right[i])};out.Equal=equal253(a,b);out.NotEqual=notEqual253(a,b)
case 254:var a,b [254]byte;for i:=0;i<254;i++{a[i]=byte(in.Left[i]);b[i]=byte(in.Right[i])};out.Equal=equal254(a,b);out.NotEqual=notEqual254(a,b)
case 255:var a,b [255]byte;for i:=0;i<255;i++{a[i]=byte(in.Left[i]);b[i]=byte(in.Right[i])};out.Equal=equal255(a,b);out.NotEqual=notEqual255(a,b)
case 256:var a,b [256]byte;for i:=0;i<256;i++{a[i]=byte(in.Left[i]);b[i]=byte(in.Right[i])};out.Equal=equal256(a,b);out.NotEqual=notEqual256(a,b)
default:panic("unsupported length")};return out}
func main(){var in []Input;if err:=json.NewDecoder(os.Stdin).Decode(&in);err!=nil{panic(err)};out:=make([]Output,0,len(in));for _,row:=range in{out=append(out,compare(row))};if err:=json.NewEncoder(os.Stdout).Encode(out);err!=nil{panic(err)}}
