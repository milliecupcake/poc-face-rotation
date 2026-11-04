# アニメ顔の横回転モデル ブラウザ実装仕様

版：1.0　作成日：2026年11月4日

## 1. 目的と実装範囲

正面の顔を左右へ回したときの、目の横幅、輪郭、顔の中心線、顎先の位置を、同じ座標系と回転式で計算する。学習用の幾何モデルであり、鼻・耳・髪・口・首の形、眼球の立体、上下の回転、遠近法は扱わない。

顔の形状は、添付された正面輪郭を手作業で左右対称に近似したもの。正面画像から3D形状を自動推定したものではない。目の位置、前後の厚み、回転軸、顎の奥行きは説明用の設計値であり、人間の平均値ではない。

実装の基準は、最後に提示した「目と輪郭を一緒に回すモデル」とする。過去の説明との対応は次のとおり。

| モデル | 表現するもの | 扱い |
| --- | --- | --- |
| 顎の点モデル | 前後の距離が横ずれに変換されること | 共通の回転式から導出 |
| 目の平面モデル | 目頭・目尻の前後差と目幅の関係 | 独立した簡易モデルとして第4節に記載 |
| 楕円断面モデル | 輪郭と、曲面上の目の変化 | 本実装の基準 |

**目の平面モデルの角度 φ と、楕円断面モデルの奥行き倍率 q は、別のパラメータである。** 最後のデモに独立した φ スライダーはない。楕円モデルでは、顔の奥行きと目の配置から、目頭と目尻の前後差が決まる。

## 2. 座標系と用語

### 2.1 顔に固定された座標

| 記号 | 意味 | 正の方向・範囲 |
| --- | --- | --- |
| x | 左右の位置 | 正面で画面右へ正 |
| y | 高さの位置 | 下へ正。頭頂0、顎先172 |
| z | 前後の位置 | 顔の前方、正面時に観察者へ近づく向きへ正 |
| θ | 顔の横回転角 | 画面右向きへ正、左向きへ負 |
| X, Y, Z | 回転後の位置 | Zが大きいほど観察者に近い |
| s | 目の左右の識別子 | 正面で画面左の目が−1、画面右の目が+1 |
| q | 前後方向の倍率 | 0より大きい。初期値1 |

左右はすべて**画面基準**とする。人物自身の右目・左目という解剖学的な呼称と混ぜない。s は回転しても変えず、「手前／奥」の表示だけを切り替える。

回転軸は x=0, z=0 を通る縦軸。首付近を想定した固定軸であり、実際の頸部運動を再現するものではない。長さの単位は任意のモデル単位で、pxへの変換は描画時に行う。

### 2.2 観察と投影

観察者は回転後の +Z 側から −Z 方向を見る。正射影を使うため、奥にあるだけでは小さくならない。目幅の左右差は、主に面の向きと、顔の向こう側へ隠れることから生じる。

JavaScriptの三角関数にはラジアンを渡す。

$$
\theta_{\mathrm{rad}}=\theta_{\mathrm{deg}}\frac{\pi}{180}
$$

## 3. 共通の回転式と顎先

以下、C=cosθ、S=sinθ と書く。

$$
\begin{aligned}
X&=xC+zS\\
Y&=y\\
Z&=-xS+zC
\end{aligned}
$$

画面上の位置は、表示倍率 k と原点 (oₓ,oᵧ) を使って次のように求める。

$$
u=o_x+kX,\qquad v=o_y+kY
$$

顎先が (0,yχ,d) にある場合は、Xχ=d sinθ。軸より後ろの点 (0,yᵦ,−dᵦ) は Xᵦ=−dᵦ sinθ となる。右を向くと前方の点は右、後方の点は左へ移る。

ただし、これは**固定された点の移動**である。後頭部や頬の「輪郭線全体」が同じ点の集合のまま移動するという意味ではない。

## 4. 目の平面モデル

この節は、前半の「目の面の傾き φ」を操作したデモに対応する。目の横方向を直線とし、目尻を目頭より後ろへ置く。

| 記号 | 定義 |
| --- | --- |
| W₀ | 正面での片目の横幅。3D空間での線分長とは異なる |
| D | 目頭に対して目尻が後ろに下がる距離 |
| φ | 目の横方向の線を、上から見たときの傾き |
| ψ | 回転角の絶対値。ψ=abs(θ) |

$$
D=W_0\tan\phi
$$

目頭から目尻への差分は、Δx=sW₀、Δz=−D。回転後の差分は次のとおり。

$$
\Delta X=sW_0\cos\theta-D\sin\theta
$$

表向きの範囲で、正面からの向きを保った横幅は次の式になる。

$$
w_s=W_0\cos\theta-sD\sin\theta
$$

手前／奥で整理すると、

$$
\begin{aligned}
w_{\mathrm{near}}&=W_0\cos\psi+D\sin\psi\\
w_{\mathrm{far}}&=W_0\cos\psi-D\sin\psi
\end{aligned}
$$

このデモでは w≤0 の面を非表示にする。したがって表示幅は max(0,w)。裏向きになった後に abs(w) を使うと、目が再び広がって見えてしまうため使わない。

この非表示判定は、独立した平面の向きによるもの。顔や鼻を含む一般的な遮蔽判定ではない。平面モデルの消失角は φ≥0 のとき ψ=90°−φ。手前の目は正面より広く投影される場合がある。

以前のデモでは目頭 |x|=20、目尻 |x|=60、W₀=40、z=70−|x|tanφ、φの初期値20°、範囲0〜40°を使った。これは最後の楕円モデルの目の配置とは異なる。

## 5. 高さごとの楕円で顔を定義する

### 5.1 断面の式

高さ y ごとに、左右半径 a(y)、前後半径 b(y)、断面中心の前後位置 c(y) を定義する。

$$
\left(\frac{x}{a(y)}\right)^2+
\left(\frac{z-c(y)}{b(y)}\right)^2=1
$$

円周パラメータ α を使うと、

$$
x=a\sin\alpha,\qquad z=c+b\cos\alpha
$$

- α=0：顔の前面中央。
- α=±π/2：正面で輪郭になる左右の点。
- α=π：後ろ側の中央。
- 顔の前半分：−π/2≤α≤π/2。

c は楕円断面の中心であり、顔表面の中心線ではない。表面の中心線の奥行きは c+b。

### 5.2 初期形状データ

この表が、最後のデモで使用した値である。a₀ と b₀ は直径ではなく半径。

| y | a₀ 左右半径 | b₀ 前後半径 | c₀ 中心の前後位置 |
| --- | --- | --- | --- |
| 0 | 0 | 0 | −5 |
| 2 | 13 | 15 | −5 |
| 8 | 28 | 32 | −4 |
| 18 | 43 | 50 | −3 |
| 30 | 54 | 62 | −2 |
| 42 | 59 | 67 | −2 |
| 60 | 62 | 70 | −1 |
| 80 | 61 | 69 | 0 |
| 100 | 57 | 63 | 2 |
| 120 | 50 | 49 | 10 |
| 135 | 44 | 32 | 24 |
| 147 | 36 | 20 | 38 |
| 157 | 23 | 11 | 49 |
| 167 | 8 | 4 | 57 |
| 172 | 0 | 0 | 60 |

補間後に奥行き倍率を適用する。

$$
a(y)=a_0(y),\qquad b(y)=q\,b_0(y),\qquad c(y)=q\,c_0(y)
$$

q は前後半径 b だけでなく、位置 c も倍率変更する。したがって顎先の奥行きも d=60q になる。b だけを変える実装では、最後のデモとは別の挙動になる。

### 5.3 補間

a₀、b₀、c₀ をそれぞれ独立に、区分的な3次Hermite補間でつなぐ。内部点の傾きには重み付き調和平均を使う。一般的な補間ライブラリと端点処理が違う場合があるため、再現には以下の定義を使う。

対象の値を fᵢ、間隔を hᵢ=yᵢ₊₁−yᵢ、区間の傾きを δᵢ=(fᵢ₊₁−fᵢ)/hᵢ とする。

- 最初の傾き m₀=δ₀、最後の傾き mₙ₋₁=δₙ₋₂。
- 内部点で δᵢ₋₁δᵢ≤0 なら mᵢ=0。
- それ以外は次式。

$$
m_i=\frac{3(h_{i-1}+h_i)}{
(2h_i+h_{i-1})/\delta_{i-1}+(h_i+2h_{i-1})/\delta_i}
$$

区間内で t=(y−yᵢ)/hᵢ とすると、

$$
f(y)=(2t^3-3t^2+1)f_i+(t^3-2t^2+t)h_i m_i
+(-2t^3+3t^2)f_{i+1}+(t^3-t^2)h_i m_{i+1}
$$

実装では a₀、b₀ の補間結果を max(0,value) とする。対象の y は0〜172。範囲外を外挿しない。

## 6. 回転後の輪郭

### 6.1 画面上の左右端

高さ y の断面を回転すると、

$$
X(\alpha)=cS+aC\sin\alpha+bS\cos\alpha
$$

ここで、

$$
R=\sqrt{a^2C^2+b^2S^2}
$$

とおけば、その高さの左右の輪郭は厳密に次の式で求まる。

$$
\boxed{X_L=cS-R,\qquad X_R=cS+R}
$$

輪郭を求めるために円周全体を探索する必要はない。JavaScriptでは R=Math.hypot(a*C,b*S) を使う。

正面では X_L=−a、X_R=a。真横では左右端は z=c−b と z=c+b の投影になる。輪郭の幅は2Rであり、一般には「正面の幅×cosθ」にはならない。例えば a=b の円断面なら、回転しても幅2aは一定。

描画時は y を0→172へ走査して左端を並べ、172→0へ走査して右端を並べ、閉じたパスにする。最後のデモの刻みは Δy=0.5。

### 6.2 今、輪郭になっている表面の点

断面上のどの点が輪郭になるかも計算できる。σ=−1 を左端、σ=+1 を右端とすると、R>0 のとき、

$$
x_\sigma=\sigma\frac{a^2C}{R},\qquad
z_\sigma=c+\sigma\frac{b^2S}{R}
$$

この点を回転すると Xσ=cS+σR になる。断面図ではこれを「● 今の輪郭の点」として表示する。

「□ 正面で輪郭だった点」は固定された (x,z)=(−a,c),(a,c) を回転して描く。回転後も●と□を同じ位置として扱ってはいけない。

頭頂と顎先では a=b=0、R=0。この場合は1点 (0,y,c) として扱い、Rで割る式を使わない。

### 6.3 表面の中心線と左右の余白

顔表面の中心線は (x,z)=(0,c+b) なので、

$$
X_F=(c+b)S
$$

この中心線から左右の輪郭までの距離は、

$$
M_L=R+bS,\qquad M_R=R-bS
$$

右向きでは右側の余白 M_R が詰まり、左向きでは左側 M_L が詰まる。これは中心線から輪郭までの幅であり、目尻から輪郭までの余白とは別の値である。

顎先は y=172、a=b=0、c=60q なので、Xχ=60q sinθ。

## 7. 楕円表面に目を配置する

### 7.1 前面の奥行き

正面絵で目の点 (x,y) を決め、|x|≤a(y)、a>0、b>0 の範囲で前面の奥行きを求める。

$$
z(x,y)=c(y)+b(y)\sqrt{1-\left(\frac{x}{a(y)}\right)^2}
$$

同じことを α=asin(x/a)、z=c+b cosα として計算してもよい。この (x,y,z) に第3節の回転式を適用する。

目は表面に貼った曲線として扱う。眼球の球面、まぶたの厚み、瞳の追視は実装しない。

### 7.2 上下のまぶた

初期値は、目頭の距離13、目の幅30、目の高さの基準88、片側の縦幅8。

$$
\begin{aligned}
x(t)&=s(13+30t),&&0\le t\le1\\
y_{\mathrm{upper}}(t)&=88-8\sin(\pi t)\\
y_{\mathrm{lower}}(t)&=88+8\sin(\pi t)
\end{aligned}
$$

各点の y に対応する断面を使って z を求める。目の中央の高さ y=88 の断面だけを、まぶた全体に流用しない。

最後のデモでは、上まぶた・下まぶたをそれぞれ160区間、161点でサンプリングしている。

### 7.3 瞳の簡略曲線

$$
x(\beta)=28s+3.2\cos\beta,\qquad
y(\beta)=88+5.6\sin\beta,\quad 0\le\beta\le2\pi
$$

こちらも各点を顔表面に貼り、回転する。100区間、101点を使う。最終デモは輪郭線だけを描いており、実際の虹彩・瞳孔の構造は表現していない。

### 7.4 平面モデルとの関係

同じ高さで目頭を xᵢ、目尻を xₒ とし、D=z(xᵢ,y)−z(xₒ,y)、W₀=|xₒ−xᵢ| と置けば、両端を結ぶ線の角度は φchord=atan(D/W₀)。両端の投影差には第4節と同じ式が成り立つ。

ただし、楕円上の目は曲線なので、途中の点が両端より外へ投影されたり、一部だけ隠れたりする。**両端の投影差と、見えている部分全体の横幅は常には一致しない。** 最終デモの目幅は第8節の方法で測る。

## 8. 目の可視判定と横幅

### 8.1 顔の向こう側へ隠れる部分

楕円表面の法線の x,z 成分は、共通の正の倍率を省いて次のように書ける。

$$
n_x=\frac{\sin\alpha}{a},\qquad n_z=\frac{\cos\alpha}{b}
$$

回転後の観察者方向成分は、

$$
F=n'_z=\frac{\cos\alpha\,C}{b}-\frac{\sin\alpha\,S}{a}
$$

- F>0：こちらを向いている。
- F=0：輪郭に接する境界。
- F<0：向こう側なので描かない。

最終デモとの互換判定は F≥−10⁻¹⁰。この許容値は現在のモデル単位を前提とする。任意の大きさへモデルそのものを変更する場合は法線を正規化してから判定する。

この方法が遮蔽判定として使えるのは、正射影・横回転のみで、各高さの断面が凸な楕円だからである。高さを混ぜる上下回転、鼻などの別の突出物、凹んだ断面を導入する場合には、そのまま一般化できない。

### 8.2 可視な線分の組み立て

各曲線をサンプリングし、可視点が連続する区間ごとに別のパスを作る。不可視区間を挟んだ点を直線でつながない。可視点が0〜1点だけの区間は線として描かない。

現状はサンプル点で区切るため、境界に小さな誤差がある。必要なら F の符号が変わった区間に二分探索を追加し、F=0 の点まで線を延ばせる。これは追加改善であり、元のデモには含まれない。

### 8.3 表示する目幅

最終デモで表示する値は「y=88 の目の横方向の線のうち、見える部分の投影幅」。上下まぶたを含む全体のバウンディングボックスではない。

1. x=s(13+30t)、y=88、0≤t≤1 を300区間、301点で生成する。
2. 表面の z と回転後の X を計算する。
3. F≥−10⁻¹⁰ の点だけを残す。
4. 残った点の max X−min X を W とする。点がなければ W=0。
5. 正面比を 100W/30 [%] として表示する。

pxで計算した場合は分母を30kにする。UIでは整数に丸め、計算中の値は丸めない。

### 8.4 手前／奥の対応

| 回転 | 手前 | 奥 |
| --- | --- | --- |
| θ>0：画面右 | s=−1 | s=+1 |
| θ<0：画面左 | s=+1 | s=−1 |
| θ=0 | 「画面左の目」と表示 | 「画面右の目」と表示 |

0°には幾何学的な手前／奥の区別がない。データ上は目の s を保持し、表示名を切り替える。

## 9. UIと描画の仕様

### 9.1 操作項目

| 項目 | 初期値 | 範囲 | 刻み |
| --- | --- | --- | --- |
| 横回転 θ | +30° | −90〜+90° | 1° |
| 奥行き倍率 q | 1.00 | 0.55〜1.35 | 0.01 |
| 正面の輪郭を重ねる | OFF | ON／OFF | — |

目の平面モデルを別途提供する場合だけ、φの操作を追加する。統合モデルにφを独立パラメータとして追加するには、目の貼り方または顔表面の定義を変更する必要がある。

### 9.2 表示内容

- 顔：回転後の輪郭、顔表面の中心線、目の高さのガイド、両目、顎先。
- 断面：y=88 の楕円を上から見た図、両目の位置、現在の輪郭の●、正面の輪郭だった□、投影幅。
- 数値：左右または手前／奥の目幅の正面比。
- 注記：正面輪郭は近似、目の位置と奥行きは仮定、鼻・耳・遠近感を省略。

上から見た断面図の表示座標は u=oₓ+kX、v=o_z+kZ とする。+Zが画面下へ向くため、観察者は下側に位置し、「見る方向 ↑」となる。この図は内部配置を示す説明図なので、裏側の目の位置も表示する。

最終デモの描画順は、正面の参考線 → 目の高さのガイド → 中心線 → 輪郭 → 目 → 輪郭上の点・顎先。パスには塗りつぶしを使わない。輪郭を最後に重ねる方法も選べるが、可視判定そのものは必要。

### 9.3 ブラウザへの移植

SVGまたはCanvasで実装できる。SVGでは輪郭を閉じた path、目の可視区間を開いた path とし、変化のたびに座標を更新する。ブラウザ標準機能だけで計算でき、3Dライブラリは必須ではない。

既存デモに出てくる `--foreground`、`--viz-series-*`、`.viz-controls` などは表示環境が提供する色・スタイルである。独立したHTMLでは、自分のCSSで定義するか置き換える。第12節の例はこれらに依存しない。

元デモのレイアウトを再現する場合、顔と断面のSVGはそれぞれ幅を計測し、高さ328pxとする。顔の倍率は min(1.55,(width−32)/205)、原点は (width/2,17)。断面の倍率は min(1.10,(width−40)/224)、原点は (width/2,137)。画面幅540px以下では縦に積む。これは表示上の値であり、モデルの計算式とは独立している。

操作は range の input と checkbox の change で反映する。SVGの固定viewBoxを使えばリサイズ時の座標再計算は不要。幅に応じた倍率を使う場合は ResizeObserver で描き直す。0px幅での初回計測を処理し、数値計算と表示倍率を混ぜない。

## 10. 入力制約と実装上の注意

| 条件 | 処理 |
| --- | --- |
| a=b=0 の頭頂・顎先 | 輪郭を1点として描く。表面法線やasinを計算しない |
| 目の点が abs(x)>a(y) | 設定不正として検出する。丸め誤差を超えて端に押し込めない |
| q≤0、非有限の値 | 入力エラーとして扱う |
| yが0〜172の範囲外 | 外挿せずエラーにする |
| asin・sqrtの丸め誤差 | 有効な入力と確認後、比を−1〜1、平方根の中を0以上に制限する |
| 可視点がない目 | 横幅0。NaNやInfinityを表示しない |
| 正面0° | 左右の幅は同じ。手前／奥のラベルを使わない |
| 真横±90° | cosθの微小な浮動小数点誤差を許容する |

上表の `abs(x)` は絶対値を表す。輪郭を作る点、まぶた、瞳、中心線の全てに同じ回転角とqを使う。元デモは入力範囲を固定しており、一部の入力検証を省略していた。以下の参照実装では、無効な値の検出を加えている。

## 11. JavaScript参照実装

以下を `face-model.js` として保存すると、描画に依存しない計算部分として利用できる。座標・サンプリング数・初期値は最終デモに対応する。返す線の座標はpx変換前のモデル単位。

```jsx
// face-model.js
globalThis.FaceModel = (() => {
  const EPS = 1e-10;
  const ROWS = [
    [0,0,0,-5], [2,13,15,-5], [8,28,32,-4], [18,43,50,-3],
    [30,54,62,-2], [42,59,67,-2], [60,62,70,-1], [80,61,69,0],
    [100,57,63,2], [120,50,49,10], [135,44,32,24], [147,36,20,38],
    [157,23,11,49], [167,8,4,57], [172,0,0,60]
  ];
  const rad = degrees => degrees * Math.PI / 180;
  const clamp = (x, lo, hi) => Math.max(lo, Math.min(hi, x));
  const slopes = [1,2,3].map(j => ROWS.map((r,i) => {
    if (i === 0) return (ROWS[1][j]-r[j])/(ROWS[1][0]-r[0]);
    if (i === ROWS.length-1) {
      return (r[j]-ROWS[i-1][j])/(r[0]-ROWS[i-1][0]);
    }
    const h0 = r[0]-ROWS[i-1][0], h1 = ROWS[i+1][0]-r[0];
    const d0 = (r[j]-ROWS[i-1][j])/h0;
    const d1 = (ROWS[i+1][j]-r[j])/h1;
    return d0*d1 <= 0 ? 0 :
      3*(h0+h1)/((2*h1+h0)/d0+(h1+2*h0)/d1);
  }));

  function checkView(degrees, q) {
    if (!Number.isFinite(degrees) || Math.abs(degrees) > 90 ||
        !Number.isFinite(q) || q <= 0) {
      throw new RangeError('degrees must be in [-90,90]; q must be positive');
    }
  }

  function shape(y, q=1) {
    if (!Number.isFinite(y) || y < 0 || y > 172 ||
        !Number.isFinite(q) || q <= 0) throw new RangeError('Invalid shape input');
    let i = 0;
    while (i < ROWS.length-2 && ROWS[i+1][0] < y) i++;
    const h = ROWS[i+1][0]-ROWS[i][0], t = (y-ROWS[i][0])/h;
    const values = [1,2,3].map((j,k) =>
      (2*t**3-3*t*t+1)*ROWS[i][j] +
      (t**3-2*t*t+t)*h*slopes[k][i] +
      (-2*t**3+3*t*t)*ROWS[i+1][j] +
      (t**3-t*t)*h*slopes[k][i+1]);
    return {a:Math.max(0,values[0]), b:q*Math.max(0,values[1]), c:q*values[2]};
  }

  function rotate(x, y, z, theta) {
    const C = Math.cos(theta), S = Math.sin(theta);
    return {X:x*C+z*S, Y:y, Z:-x*S+z*C};
  }

  function bounds(y, theta, q=1) {
    const {a,b,c} = shape(y,q), C = Math.cos(theta), S = Math.sin(theta);
    const R = Math.hypot(a*C,b*S);
    return {left:c*S-R, right:c*S+R, center:(c+b)*S, R};
  }

  // Returns local coordinates of the current left/right contour point.
  function contourPoint(y, sigma, theta, q=1) {
    const {a,b,c} = shape(y,q), C = Math.cos(theta), S = Math.sin(theta);
    const R = Math.hypot(a*C,b*S);
    if (R < EPS) return {x:0, y, z:c};
    return {x:sigma*a*a*C/R, y, z:c+sigma*b*b*S/R};
  }

  function surfacePoint(x, y, theta, q=1) {
    const {a,b,c} = shape(y,q);
    if (!Number.isFinite(x) || a <= EPS || b <= EPS || Math.abs(x) > a+EPS) {
      throw new RangeError('Eye point must lie on a nondegenerate front slice');
    }
    const alpha = Math.asin(clamp(x/a,-1,1));
    const z = c+b*Math.cos(alpha);
    const facing = Math.cos(alpha)*Math.cos(theta)/b -
                   Math.sin(alpha)*Math.sin(theta)/a;
    return {...rotate(x,y,z,theta), facing, visible:facing >= -EPS};
  }

  function visibleRuns(points) {
    const runs = [];
    let run = [];
    for (const p of [...points,null]) {
      if (p && p.visible) run.push([p.X,p.Y]);
      else {
        if (run.length > 1) runs.push(run);
        run = [];
      }
    }
    return runs;
  }

  function eye(side, theta, q=1) {
    const runs = [];
    for (const vertical of [-1,1]) {
      const points = [];
      for (let i=0;i<=160;i++) {
        const t=i/160;
        points.push(surfacePoint(side*(13+30*t),
          88+vertical*8*Math.sin(Math.PI*t),theta,q));
      }
      runs.push(...visibleRuns(points));
    }
    const pupil = [];
    for (let i=0;i<=100;i++) {
      const beta=i/100*2*Math.PI;
      pupil.push(surfacePoint(28*side+3.2*Math.cos(beta),
        88+5.6*Math.sin(beta),theta,q));
    }
    runs.push(...visibleRuns(pupil));
    const xs = [];
    for (let i=0;i<=300;i++) {
      const p=surfacePoint(side*(13+i/10),88,theta,q);
      if (p.visible) xs.push(p.X);
    }
    const width=xs.length ? Math.max(...xs)-Math.min(...xs) : 0;
    return {side, runs, width, percent:100*width/30};
  }

  function buildScene(degrees=30,q=1) {
    checkView(degrees,q);
    const theta=rad(degrees), outline=[], centerline=[], reference=[];
    for (let i=0;i<=344;i++) {
      const y=i/2;
      outline.push([bounds(y,theta,q).left,y]);
      reference.push([-shape(y,1).a,y]);
    }
    for (let i=344;i>=0;i--) {
      const y=i/2;
      outline.push([bounds(y,theta,q).right,y]);
      reference.push([shape(y,1).a,y]);
    }
    for (let y=0;y<=172;y++) centerline.push([bounds(y,theta,q).center,y]);
    const eyes=[eye(-1,theta,q),eye(1,theta,q)];
    for (const e of eyes) {
      e.role=degrees === 0 ? (e.side === -1 ? 'left' : 'right') :
        (e.side*Math.sign(degrees) < 0 ? 'near' : 'far');
    }
    const slice=bounds(88,theta,q);
    return {
      degrees,q,theta,outline,reference,centerline,eyes,
      chin:[60*q*Math.sin(theta),172],
      eyeLevel:[[slice.left,88],[slice.right,88]]
    };
  }

  // Independent planar-eye model: angles are in degrees here.
  function planarEyeWidths(degrees,phiDegrees,W0=40) {
    checkView(degrees,1);
    if (!Number.isFinite(phiDegrees) || phiDegrees < 0 || phiDegrees >= 90 ||
        !Number.isFinite(W0) || W0 <= 0) throw new RangeError('Invalid planar eye input');
    const psi=rad(Math.abs(degrees)), D=W0*Math.tan(rad(phiDegrees));
    const near=W0*Math.cos(psi)+D*Math.sin(psi);
    const far=W0*Math.cos(psi)-D*Math.sin(psi);
    return {D,near:Math.max(0,near),far:Math.max(0,far)};
  }

  return {shape,rad,rotate,bounds,contourPoint,surfacePoint,visibleRuns,
          eye,buildScene,planarEyeWidths};
})();
```

APIの角度単位は、`buildScene`、`planarEyeWidths`、`rad` の引数が度。`rotate`、`bounds`、`contourPoint`、`surfacePoint`、`eye` の theta はラジアン。

## 12. ブラウザで動かす最小の描画例

第11節のコードを `face-model.js`、以下を同じディレクトリの `index.html` に保存して開く。この例は顔の表示・左右回転・奥行き・正面の参考線・目幅表示までを含む。断面図は含めない。断面図を追加する場合は第6.2節、第9.2節と `contourPoint` を使う。

フレームワーク、パッケージのインストール、外部CDN、モジュール読み込みは不要。

```html
<!doctype html>
<html lang="ja">
<meta charset="utf-8">
<meta name="viewport" content="width=device-width, initial-scale=1">
<title>顔の横回転</title>
<style>
  body { font:16px/1.6 system-ui,sans-serif; max-width:680px; margin:24px auto; padding:0 16px; }
  label { display:block; margin:12px 0; }
  input[type=range] { display:block; width:100%; }
  svg { display:block; width:100%; max-height:520px; }
  path { fill:none; stroke:#252525; stroke-width:1.2; stroke-linecap:round; stroke-linejoin:round; }
  .guide { stroke:#999; stroke-width:0.6; stroke-dasharray:2 3; }
  .reference { stroke:#bbb; stroke-width:0.7; stroke-dasharray:3 3; }
  .near { stroke:#087c73; }
  .far { stroke:#9a399e; }
</style>
<label>横回転 <output id="angleValue"></output>
  <input id="angle" type="range" min="-90" max="90" step="1" value="30">
</label>
<label>奥行き <output id="depthValue"></output>
  <input id="depth" type="range" min="0.55" max="1.35" step="0.01" value="1">
</label>
<label><input id="reference" type="checkbox">正面の輪郭を重ねる</label>
<svg id="face" viewBox="-120 -12 240 202" role="img" aria-label="横に回転する顔"></svg>
<p id="status" aria-live="polite"></p>
<script src="./face-model.js"></script>
<script>
  const byId=id=>document.getElementById(id);
  const path=(points,cls='',closed=false)=>
    `<path class="${cls}" d="M${points.map(p=>p.join(',')).join(' L')}${closed?' Z':''}"/>`;
  function draw() {
    const degrees=Number(byId('angle').value), q=Number(byId('depth').value);
    const scene=FaceModel.buildScene(degrees,q);
    let markup=byId('reference').checked ? path(scene.reference,'reference',true) : '';
    markup+=path(scene.eyeLevel,'guide')+path(scene.centerline,'guide');
    markup+=path(scene.outline,'',true);
    for (const eye of scene.eyes) {
      const cls=['near','left'].includes(eye.role)?'near':'far';
      for (const run of eye.runs) markup+=path(run,cls);
    }
    markup+=`<circle cx="${scene.chin[0]}" cy="172" r="1.7" fill="#252525"/>`;
    byId('face').innerHTML=markup;
    byId('angleValue').textContent=degrees===0?'正面 0°':
      `${degrees<0?'左':'右'}${Math.abs(degrees)}°`;
    byId('depthValue').textContent=q.toFixed(2)+'倍';
    const names={left:'画面左',right:'画面右',near:'手前',far:'奥'};
    byId('status').textContent='見える目幅（正面比）：'+scene.eyes.map(e=>
      `${names[e.role]} ${Math.round(e.percent)}%`).join(' ／ ');
  }
  byId('angle').addEventListener('input',draw);
  byId('depth').addEventListener('input',draw);
  byId('reference').addEventListener('change',draw);
  draw();
</script>
</html>
```

この描画例は単独ブラウザ用に表示を簡略化している。幾何計算は参照実装と共通だが、最後のデモのレイアウトを完全に複製するものではない。

## 13. 検証条件

### 13.1 幾何学的な条件

1. θ=0 で X=x、顎先の横位置0、左右の目幅が100%。
2. θ=0 では q を変更しても輪郭・中心線・目の投影位置が変わらない。
3. +θ と −θ の輪郭が左右反転で一致する。正確には L(−θ)=−R(θ)、R(−θ)=−L(θ)。
4. y=172 の顎先の横位置が 60q sinθ と一致する。
5. θ=±90°で奥の目の表示幅が0になる。
6. ●の座標を回転したXが、解析式の X_L または X_R に一致する。
7. 目の全サンプルが有効な前面断面上にあり、NaN、Infinity、負の表示幅がない。
8. 独立した平面モデルで φ=0 のとき、左右の幅が W₀cosθ で一致する。

### 13.2 最終デモと照合する数値

q=1、目の配置とサンプリングは本仕様の初期値。目幅は整数に丸めたUI表示。

| 回転角の絶対値 | 手前の目 正面比 | 奥の目 正面比 | 顎先の横ずれの絶対値 |
| --- | --- | --- | --- |
| 0° | 100% | 100% | 0 |
| 30° | 118% | 55% | 30 |
| 60° | 105% | 7% | 約51.9615 |
| 90° | 63% | 0% | 60 |

0°のUIラベルは「画面左／画面右」。表の手前／奥の列名は比較のための表記。q=0.55 と1.35でも前項の検証を行う。数値許容誤差は、モデル座標の比較では概ね10⁻⁸を基準にする。

### 13.3 表示の確認

- 320px程度の幅でも操作ラベルや図が切れない。
- キーボードでスライダーを操作でき、現在値を読める。
- 目の色と「手前／奥」のラベルが一致する。
- 不可視区間の前後を直線でつないでいない。
- 正面参考線は回転角・qを変えても動かない。
- スライダーを端から端へ動かしても、裏側の目が再出現しない。

## 14. モデルの限界と拡張時の判断

正面の輪郭だけでは b(y) と c(y) は一意に決まらない。同じ正面形から、異なる斜め顔・横顔を設計できる。今回の数値はそのうちの一例であり、好きな絵柄に合わせて横顔を見ながら調整する。

各断面は凸な楕円なので、こめかみのくぼみや局所的な頬の凹凸、鼻の遮蔽は表現していない。必要になったときは、断面を別の曲線に変更するか、メッシュなどで形と遮蔽を計算する。その場合も「同じ点を回す計算」と「輪郭として見える点を選ぶ計算」を分ける考え方は維持できる。

アニメ的な目の大きさの補正や、特定の向きで輪郭を描き替える処理は、本仕様にはまだ含まない。追加する場合は、幾何計算の結果に対する絵柄の補正として別に定義し、回転式そのものと混ぜない。