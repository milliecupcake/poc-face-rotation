//! アニメ顔の横回転モデル：描画に依存しない幾何計算部分（仕様 第3〜8節・第11節）。
//!
//! 返す座標はすべて px 変換前のモデル単位。
//! 角度単位：`build_scene` / `planar_eye_widths` / `rad` の引数は度、
//! それ以外の `theta` はラジアン。

use std::fmt;
use std::sync::LazyLock;

pub const EPS: f64 = 1e-10;
pub const Y_MAX: f64 = 172.0;
/// 目の高さ（目の中心・瞳の中心の y）。参考画像（顔の高さ 961px 中、瞳 449px）から 80.4。
/// 仕様 第7.2節の初期値は 88。
pub const EYE_Y: f64 = 80.4;
/// 目の大きさの倍率。仕様 第7.2〜7.3節の目（幅30・縦幅8・瞳3.2×5.6）に対する倍率。
/// 参考画像の虹彩の大きさから 1.35。
pub const EYE_SCALE: f64 = 1.35;
/// 瞳の中心の |x| の初期値。仕様の初期値は 28。
pub const EYE_CENTER_X: f64 = 38.5;
/// 目の画像素材（まつ毛の先）が y≈79 の輪郭 a(y) の内側に収まる、瞳の |x| の最大値。
/// これより外では、画像の目の外側の端が輪郭で欠ける。
pub const EYE_IMAGE_FIT_MAX: f64 = 33.5;
/// 瞳の中心の |x| として受け付ける範囲。線の目（目頭〜目尻、まぶた、瞳）が
/// 全体に有効な前面断面上に収まる範囲。
pub const EYE_CENTER_X_MIN: f64 = 22.0;
pub const EYE_CENTER_X_MAX: f64 = 44.0;

/// 左右半径 a₀ の節点 (y, a₀)。添付の正面輪郭画像（476×556 px、頭頂 28px〜顎先 528px を
/// y = 0〜172 に対応、1px = 0.344 単位）から線の中心を測って作った左右対称の値。
/// 仕様 第5.2節の a₀ 列を置き換えるもの。
const A_ROWS: [[f64; 2]; 31] = [
    [0.0, 0.0],
    [0.5, 5.70],
    [1.0, 10.05],
    [2.0, 16.50],
    [3.0, 20.01],
    [5.0, 25.31],
    [8.0, 31.82],
    [12.0, 38.81],
    [18.0, 47.07],
    [26.0, 54.38],
    [36.0, 61.52],
    [48.0, 67.42],
    [60.0, 69.83],
    [68.0, 70.52],
    [74.0, 70.52],
    [80.0, 69.32],
    [90.0, 66.74],
    [100.0, 63.98],
    [110.0, 60.89],
    [120.0, 57.28],
    [128.0, 53.49],
    [134.0, 50.40],
    [138.0, 47.93],
    [142.0, 44.10],
    [148.0, 37.59],
    [154.0, 30.61],
    [160.0, 22.28],
    [165.0, 14.17],
    [169.0, 7.20],
    [171.0, 3.34],
    [172.0, 0.0],
];

/// 前後半径 b₀ と断面中心 c₀ の節点 (y, b₀, c₀)。
///
/// 45° に回したときの輪郭の手描き指定（V字断面 φ = 20°、q = 1）に合うよう、高さごとに
/// 左右端の2式 X_R = (a + z_side)/√2、X_L = (z_side − √(a² + bb²))/√2 を b, c について解いた値。
/// 正面輪郭 a₀ は変えていないので、正面の見た目は変わらない。
/// 頭頂付近（y ≤ 8）は指定の線がモデルの頭頂 y = 0 より上にあり解けないため、
/// y = 12 と同じ比 b ≈ 1.4a で丸くつないでいる。顎先 (172) は c = 60、b = 0（仕様と同じ）。
/// 仕様 第5.2節の値は `SPEC_BC_ROWS` に残してある。
const BC_ROWS: [[f64; 3]; 32] = [
    [0.0, 0.0, -1.30],
    [2.0, 23.10, -1.28],
    [4.0, 31.96, -1.47],
    [8.0, 44.55, -1.38],
    [12.0, 54.29, -1.06],
    [18.0, 60.28, -0.49],
    [24.0, 64.97, -0.05],
    [30.0, 68.22, 0.01],
    [40.0, 71.35, -0.01],
    [50.0, 72.98, 0.44],
    [60.0, 73.15, 0.74],
    [70.0, 72.44, 0.70],
    [80.0, 70.53, 0.54],
    [90.0, 67.70, 0.59],
    [100.0, 65.05, 2.08],
    [110.0, 61.94, 5.37],
    [116.0, 59.30, 8.39],
    [120.0, 57.50, 10.93],
    [124.0, 55.32, 13.33],
    [128.0, 51.26, 17.09],
    [132.0, 47.39, 19.55],
    [136.0, 44.28, 21.09],
    [140.0, 41.33, 22.58],
    [144.0, 38.78, 24.31],
    [148.0, 34.90, 27.61],
    [152.0, 30.16, 31.55],
    [156.0, 25.27, 35.77],
    [160.0, 20.42, 40.23],
    [163.0, 15.82, 44.98],
    [166.0, 9.18, 51.54],
    [169.0, 5.00, 55.70],
    [172.0, 0.0, 60.0],
];

/// 仕様 第5.2節の b₀, c₀（参考用）。
#[allow(dead_code)]
const SPEC_BC_ROWS: [[f64; 3]; 15] = [
    [0.0, 0.0, -5.0],
    [2.0, 15.0, -5.0],
    [8.0, 32.0, -4.0],
    [18.0, 50.0, -3.0],
    [30.0, 62.0, -2.0],
    [42.0, 67.0, -2.0],
    [60.0, 70.0, -1.0],
    [80.0, 69.0, 0.0],
    [100.0, 63.0, 2.0],
    [120.0, 49.0, 10.0],
    [135.0, 32.0, 24.0],
    [147.0, 20.0, 38.0],
    [157.0, 11.0, 49.0],
    [167.0, 4.0, 57.0],
    [172.0, 0.0, 60.0],
];

// 真横（|θ| = 90°、q = 1）の輪郭の手描き指定。各行は [row, X, Y] で、row は断面モデルの
// 高さ y のどの輪郭点がこの点へ動くかを表す。X, Y はモデル単位で、指定画像の後頭部の点
// (y = EYE_Y) と顎先の点を基準に測った。row ≥ 30 はおおむね row = Y、
// 頭頂側は線の長さに沿って割り当てている（指定の頭頂は Y ≈ −10.8 で、モデルの y = 0 より上）。

/// 額〜鼻〜唇〜顎先の線。
const PROFILE_FRONT: [[f64; 3]; 70] = [
    [0.00, -0.79, -10.83],
    [0.64, 1.13, -10.82],
    [1.88, 4.82, -10.43],
    [2.73, 7.37, -10.37],
    [3.41, 9.36, -10.02],
    [5.07, 14.29, -9.50],
    [6.42, 18.26, -8.74],
    [6.83, 19.50, -8.62],
    [7.23, 20.65, -8.31],
    [9.06, 26.02, -7.32],
    [11.93, 34.33, -5.13],
    [13.26, 38.08, -3.86],
    [15.38, 43.79, -1.09],
    [16.71, 47.24, 0.93],
    [19.22, 53.36, 5.30],
    [23.30, 62.80, 13.03],
    [24.58, 65.54, 15.73],
    [25.61, 67.43, 18.16],
    [26.62, 68.97, 20.74],
    [27.48, 70.11, 23.06],
    [29.20, 71.91, 27.89],
    [38.01, 74.93, 38.01],
    [43.90, 76.25, 43.90],
    [46.94, 76.68, 46.94],
    [48.61, 76.79, 48.61],
    [50.61, 77.14, 50.61],
    [52.96, 77.18, 52.96],
    [54.60, 77.37, 54.60],
    [60.07, 77.48, 60.07],
    [61.67, 77.27, 61.67],
    [65.44, 76.01, 65.44],
    [72.76, 72.99, 72.76],
    [78.32, 70.33, 78.32],
    [81.14, 69.38, 81.14],
    [85.26, 68.46, 85.26],
    [86.71, 68.34, 86.71],
    [88.72, 67.99, 88.72],
    [92.77, 67.98, 92.77],
    [96.77, 68.68, 96.77],
    [102.90, 70.45, 102.90],
    [124.04, 77.44, 124.04],
    [125.84, 77.74, 125.84],
    [131.24, 79.20, 131.24],
    [132.90, 79.27, 132.90],
    [133.84, 78.94, 133.84],
    [134.77, 78.11, 134.77],
    [136.52, 75.62, 136.52],
    [138.19, 73.52, 138.19],
    [139.10, 72.74, 139.10],
    [139.86, 72.50, 139.86],
    [140.69, 72.46, 140.69],
    [142.67, 72.85, 142.67],
    [143.89, 72.99, 143.89],
    [144.45, 72.93, 144.45],
    [144.92, 72.62, 144.92],
    [146.81, 70.21, 146.81],
    [147.72, 69.35, 147.72],
    [149.40, 68.22, 149.40],
    [152.24, 66.79, 152.24],
    [155.04, 65.79, 155.04],
    [156.41, 65.46, 156.41],
    [158.06, 65.31, 158.06],
    [163.32, 64.01, 163.32],
    [168.84, 62.33, 168.84],
    [169.36, 62.07, 169.36],
    [169.63, 61.81, 169.63],
    [170.02, 60.94, 170.02],
    [170.39, 60.51, 170.39],
    [171.46, 60.05, 171.46],
    [172.00, 60.00, 172.00],
];

/// 後頭部〜うなじの線。
const PROFILE_BACK: [[f64; 3]; 57] = [
    [0.00, -0.79, -10.83],
    [0.76, -2.93, -10.82],
    [1.93, -6.20, -10.43],
    [2.60, -8.09, -10.34],
    [4.63, -13.72, -9.46],
    [5.34, -15.65, -8.95],
    [6.29, -18.30, -8.62],
    [7.29, -21.01, -7.92],
    [7.80, -22.42, -7.71],
    [8.50, -24.31, -7.10],
    [9.57, -27.23, -6.39],
    [12.54, -35.19, -3.89],
    [13.10, -36.68, -3.35],
    [13.88, -38.65, -2.42],
    [15.93, -43.71, 0.38],
    [18.29, -49.35, 3.82],
    [19.43, -52.03, 5.62],
    [20.23, -53.83, 6.97],
    [20.99, -55.33, 8.46],
    [22.43, -57.70, 11.77],
    [22.79, -58.16, 12.65],
    [23.57, -58.93, 14.69],
    [25.15, -61.50, 18.34],
    [27.31, -64.81, 23.41],
    [29.15, -67.39, 27.92],
    [31.93, -69.09, 31.93],
    [34.95, -70.09, 34.95],
    [40.20, -71.39, 40.20],
    [42.58, -71.86, 42.58],
    [44.03, -71.98, 44.03],
    [45.97, -72.50, 45.97],
    [61.82, -72.53, 61.82],
    [64.88, -72.13, 64.88],
    [67.65, -72.08, 67.65],
    [69.22, -71.74, 69.22],
    [71.33, -71.65, 71.33],
    [72.91, -71.31, 72.91],
    [74.36, -71.19, 74.36],
    [83.97, -69.01, 83.97],
    [88.85, -67.34, 88.85],
    [92.11, -66.39, 92.11],
    [94.53, -65.50, 94.53],
    [98.52, -63.74, 98.52],
    [101.55, -61.08, 103.43],
    [103.71, -58.18, 107.80],
    [105.82, -55.02, 111.85],
    [107.05, -52.91, 113.98],
    [107.82, -51.49, 115.20],
    [108.94, -49.67, 117.23],
    [112.06, -44.08, 122.37],
    [112.99, -42.31, 123.77],
    [113.81, -40.58, 124.76],
    [117.81, -31.93, 129.22],
    [119.21, -29.07, 131.12],
    [120.73, -26.13, 133.34],
    [122.88, -22.22, 136.82],
    [124.00, -20.28, 138.74],
];

/// あごの裏の線。
const PROFILE_JAW: [[f64; 3]; 23] = [
    [124.00, -15.14, 123.94],
    [124.69, -14.73, 124.63],
    [125.63, -14.43, 125.57],
    [131.21, -11.85, 131.15],
    [135.51, -9.30, 135.46],
    [137.02, -8.28, 136.97],
    [138.61, -6.95, 138.56],
    [140.72, -4.84, 140.68],
    [144.62, -0.24, 144.58],
    [147.31, 3.35, 147.28],
    [151.57, 9.73, 151.54],
    [154.25, 14.19, 154.22],
    [157.06, 19.24, 157.04],
    [160.12, 25.47, 160.10],
    [163.66, 33.43, 163.64],
    [165.86, 39.15, 165.86],
    [166.74, 41.79, 166.73],
    [167.38, 44.52, 167.38],
    [168.05, 46.60, 168.04],
    [170.83, 57.87, 170.83],
    [171.24, 58.77, 171.24],
    [171.76, 59.40, 171.76],
    [172.00, 60.00, 172.00],
];

/// 左端の輪郭のうち、真横で `PROFILE_BACK` の終わり（うなじ）と `PROFILE_JAW` の始まり
/// （あごの裏）に分かれる高さ。真横ではこの2点の間が首の開口になる。
const PROFILE_SPLIT_ROW: f64 = 124.0;

/// 手描きの横顔を効かせ始める |θ|（度）。45° の輪郭の指定を保つため、ここまでは 0。
const PROFILE_START_DEGREES: f64 = 45.0;

/// 鼻の線の手描き指定 [z, y]。鼻筋の付け根から鼻先を通って鼻の下まで、q = 1 の値。
/// 顔の中心の面 (x = 0) にある線で、回転で sinθ に比例して横へ張り出すので、
/// 正面では縦の直線、真横ではこの形になる。両端は `PROFILE_FRONT` の線の上にある。
const NOSE_PROFILE: [[f64; 2]; 27] = [
    [70.22, 78.65],
    [70.23, 79.60],
    [69.83, 82.19],
    [69.76, 83.15],
    [69.75, 85.72],
    [69.88, 87.06],
    [70.31, 89.46],
    [70.95, 91.58],
    [71.86, 93.78],
    [72.84, 95.56],
    [74.18, 97.39],
    [75.69, 99.15],
    [78.48, 101.95],
    [80.94, 104.15],
    [83.49, 106.14],
    [85.21, 107.27],
    [89.40, 109.49],
    [89.93, 109.60],
    [90.23, 109.50],
    [89.80, 109.89],
    [88.41, 113.63],
    [87.52, 115.45],
    [86.74, 116.71],
    [81.55, 122.90],
    [78.14, 126.33],
    [77.84, 126.81],
    [78.01, 126.85],
];

/// 鼻の下に隠す顔の線の幅。鼻の線から、顔の線よりこの分だけ奥までを塗りつぶす。
const NOSE_MASK_DEPTH: f64 = 3.0;

/// 真横の輪郭へ寄せる重み。|θ| ≤ 45° で 0、90° で 1 になるなめらかな階段。
fn profile_weight(degrees: f64) -> f64 {
    let t = ((degrees.abs() - PROFILE_START_DEGREES) / (90.0 - PROFILE_START_DEGREES)).clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}

/// 正面（θ = 0）の頭の上部の手描き指定 [row, 半幅, Y]。左右の線を平均して対称にしたもの。
/// row は `PROFILE_FRONT` などと同じく、どの高さの輪郭点がこの点へ動くかを表す。
/// 指定の頭頂は Y ≈ −12.0 だったが、どの角度でも同じ高さになるよう −10.83 に合わせている。
/// 最後の点は a(75) の上にあり、ここから下の輪郭は `A_ROWS` のまま。
const FRONTAL_TOP: [[f64; 3]; 33] = [
    [0.00, 0.00, -10.83],
    [2.05, 4.77, -10.58],
    [4.46, 10.39, -10.16],
    [6.51, 15.12, -9.57],
    [7.99, 18.53, -9.00],
    [9.47, 21.91, -8.28],
    [11.14, 25.66, -7.26],
    [12.62, 28.94, -6.18],
    [13.73, 31.34, -5.21],
    [15.21, 34.41, -3.63],
    [17.06, 38.09, -1.38],
    [19.09, 41.97, 1.36],
    [21.12, 45.72, 4.25],
    [23.34, 49.69, 7.57],
    [25.19, 52.87, 10.47],
    [27.03, 55.91, 13.51],
    [28.32, 57.91, 15.76],
    [29.60, 59.70, 18.15],
    [31.07, 61.58, 21.01],
    [32.71, 63.47, 24.34],
    [34.54, 65.37, 28.15],
    [36.36, 67.07, 32.05],
    [38.00, 68.43, 35.62],
    [39.64, 69.61, 39.26],
    [42.60, 70.44, 42.60],
    [44.73, 70.84, 44.73],
    [46.45, 71.05, 46.45],
    [52.08, 71.42, 52.08],
    [56.86, 71.52, 56.86],
    [62.08, 71.44, 62.08],
    [69.89, 71.02, 69.89],
    [72.90, 70.74, 72.90],
    [75.00, 70.46, 75.00],
];

/// 頭の上部を正面の指定に寄せる重み。正面で 1、|θ| ≥ 45° で 0。
fn frontal_weight(degrees: f64) -> f64 {
    let t = (degrees.abs() / PROFILE_START_DEGREES).clamp(0.0, 1.0);
    1.0 - t * t * (3.0 - 2.0 * t)
}

/// 頭頂を持ち上げる高さの範囲。y がこれより下の輪郭は動かさない。
const TOP_LIFT_RANGE: f64 = 40.0;

/// 高さ row の輪郭を持ち上げる量。頭頂で最大、`TOP_LIFT_RANGE` で 0。
/// モデルの頭頂 y = 0 を、どの角度でも真横の指定の高さ（`PROFILE_FRONT` の始点）に描く。
fn top_lift(row: f64) -> f64 {
    let t = (1.0 - row / TOP_LIFT_RANGE).max(0.0);
    -PROFILE_FRONT[0][2] * t * t
}

/// 表の row での (X, Y) を線形補間で返す。
fn profile_at(table: &[[f64; 3]], row: f64) -> (f64, f64) {
    let i = table.partition_point(|r| r[0] < row).clamp(1, table.len() - 1);
    let (p, n) = (table[i - 1], table[i]);
    let t = ((row - p[0]) / (n[0] - p[0])).clamp(0.0, 1.0);
    (p[1] + (n[1] - p[1]) * t, p[2] + (n[2] - p[2]) * t)
}

/// 区分3次Hermite補間の1系列（第5.3節）。内部点の傾きは重み付き調和平均。
struct Curve {
    ys: Vec<f64>,
    fs: Vec<f64>,
    ms: Vec<f64>,
}

impl Curve {
    fn new(points: impl IntoIterator<Item = (f64, f64)>) -> Self {
        let (ys, fs): (Vec<f64>, Vec<f64>) = points.into_iter().unzip();
        let n = ys.len();
        let ms = (0..n)
            .map(|i| {
                if i == 0 {
                    (fs[1] - fs[0]) / (ys[1] - ys[0])
                } else if i == n - 1 {
                    (fs[i] - fs[i - 1]) / (ys[i] - ys[i - 1])
                } else {
                    let h0 = ys[i] - ys[i - 1];
                    let h1 = ys[i + 1] - ys[i];
                    let d0 = (fs[i] - fs[i - 1]) / h0;
                    let d1 = (fs[i + 1] - fs[i]) / h1;
                    if d0 * d1 <= 0.0 {
                        0.0
                    } else {
                        3.0 * (h0 + h1) / ((2.0 * h1 + h0) / d0 + (h1 + 2.0 * h0) / d1)
                    }
                }
            })
            .collect();
        Self { ys, fs, ms }
    }

    fn eval(&self, y: f64) -> f64 {
        let (ys, fs, ms) = (&self.ys, &self.fs, &self.ms);
        let mut i = 0;
        while i < ys.len() - 2 && ys[i + 1] < y {
            i += 1;
        }
        let h = ys[i + 1] - ys[i];
        let t = (y - ys[i]) / h;
        let (t2, t3) = (t * t, t * t * t);
        (2.0 * t3 - 3.0 * t2 + 1.0) * fs[i]
            + (t3 - 2.0 * t2 + t) * h * ms[i]
            + (-2.0 * t3 + 3.0 * t2) * fs[i + 1]
            + (t3 - t2) * h * ms[i + 1]
    }
}

static CURVES: LazyLock<[Curve; 3]> = LazyLock::new(|| {
    [
        Curve::new(A_ROWS.iter().map(|r| (r[0], r[1]))),
        Curve::new(BC_ROWS.iter().map(|r| (r[0], r[1]))),
        Curve::new(BC_ROWS.iter().map(|r| (r[0], r[2]))),
    ]
});

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ModelError {
    /// 回転角が [-90, 90] 外、または q が正の有限値でない。
    InvalidView,
    /// y が [0, 172] 外、または非有限値。
    InvalidShapeInput,
    /// 目の点が有効な前面断面上にない（|x| > a(y) など）。
    EyeOffSurface,
    /// 平面モデルの入力が不正。
    InvalidPlanarInput,
    /// 目の間隔（瞳の中心の |x|）が範囲外、または非有限値。
    InvalidEyeLayout,
    /// 断面の形のパラメータ（V字の傾き）が範囲外。
    InvalidHead,
}

impl fmt::Display for ModelError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::InvalidView => "回転角は −90〜90°、奥行き倍率 q は正の値にしてください",
            Self::InvalidShapeInput => "高さ y は 0〜172 の範囲で指定してください",
            Self::EyeOffSurface => "目の点が顔の前面断面上にありません",
            Self::InvalidPlanarInput => "平面モデルの入力が不正です",
            Self::InvalidEyeLayout => "目の間隔は 22〜44 の範囲で指定してください",
            Self::InvalidHead => "V字の傾きは 0〜40° の範囲で指定してください",
        })
    }
}

impl std::error::Error for ModelError {}

pub type Result<T> = std::result::Result<T, ModelError>;

pub fn rad(degrees: f64) -> f64 {
    degrees * std::f64::consts::PI / 180.0
}

fn check_view(degrees: f64, q: f64) -> Result<()> {
    if !degrees.is_finite() || degrees.abs() > 90.0 || !q.is_finite() || q <= 0.0 {
        return Err(ModelError::InvalidView);
    }
    Ok(())
}

fn check_q(q: f64) -> Result<()> {
    if !q.is_finite() || q <= 0.0 {
        return Err(ModelError::InvalidView);
    }
    Ok(())
}

/// 高さ y の基本形状。a: 左右半径、b: 前後半径、c: 楕円断面の中心の前後位置（第5節）。
/// 断面の前面の形（`Front`）によらず、表面の中心線 c + b と後ろ端 c − b はこの値で決まる。
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Shape {
    pub a: f64,
    pub b: f64,
    pub c: f64,
}

pub fn shape(y: f64, q: f64) -> Result<Shape> {
    if !y.is_finite() || !(0.0..=Y_MAX).contains(&y) {
        return Err(ModelError::InvalidShapeInput);
    }
    check_q(q)?;
    let v = CURVES.each_ref().map(|c| c.eval(y));
    Ok(Shape {
        a: v[0].max(0.0),
        b: q * v[1].max(0.0),
        c: q * v[2],
    })
}

/// 断面の前半分の形。
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Front {
    /// 仕様どおりの楕円（第5節）。頭が球に近い動きになる。
    Ellipse,
    /// ゆるやかなV字。左右の角 (±a) から中心線へ向かう2枚の平面。
    /// `phi` は上から見たときの面の傾き（ラジアン、q = 1 のとき）。後ろ半分は丸い楕円。
    Wedge { phi: f64 },
}

/// 頭の形のパラメータ。
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Head {
    /// 前後方向の倍率（第2.1節）。
    pub q: f64,
    pub front: Front,
}

impl Head {
    pub fn ellipse(q: f64) -> Self {
        Self { q, front: Front::Ellipse }
    }
    pub fn wedge(q: f64, phi_degrees: f64) -> Self {
        Self { q, front: Front::Wedge { phi: rad(phi_degrees) } }
    }
}

/// V字の傾きとして受け付ける範囲（度）。
pub const WEDGE_PHI_MIN: f64 = 0.0;
pub const WEDGE_PHI_MAX: f64 = 40.0;
/// V字の傾きの初期値（度）。
pub const WEDGE_PHI_DEFAULT: f64 = 20.0;

fn check_head(head: Head) -> Result<()> {
    check_q(head.q)?;
    if let Front::Wedge { phi } = head.front {
        if !phi.is_finite() || phi < rad(WEDGE_PHI_MIN) - EPS || phi > rad(WEDGE_PHI_MAX) + EPS {
            return Err(ModelError::InvalidHead);
        }
    }
    Ok(())
}

/// 高さ y の断面。上から見て、前半分（楕円またはV字）と後ろ半分（楕円）からなる凸な形。
///
/// - 左右の角（正面での輪郭）: (±a, z_side)
/// - 前の先端（表面の中心線）: (0, z_tip)、z_tip = z_side + d
/// - 後ろ端: (0, z_side − bb)
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Section {
    pub a: f64,
    pub z_side: f64,
    /// 左右の角から前の先端までの奥行き。
    pub d: f64,
    /// 左右の角から後ろ端までの奥行き（後ろ半分の楕円の前後半径）。
    pub bb: f64,
    pub z_tip: f64,
    pub front: Front,
}

/// 高さ y の断面を作る。
///
/// 楕円：z_side = c、d = bb = b（第5節と同じ）。
/// V字：表面の中心線 c + b と後ろ端 c − b は楕円と同じ位置に保ち、
/// 前面の奥行きを d = min(q·a·tanφ, b) とする（後ろの丸みが前より浅くならないよう b で頭打ち）。
pub fn section(y: f64, head: Head) -> Result<Section> {
    check_head(head)?;
    let Shape { a, b, c } = shape(y, head.q)?;
    Ok(match head.front {
        Front::Ellipse => Section { a, z_side: c, d: b, bb: b, z_tip: c + b, front: head.front },
        Front::Wedge { phi } => {
            let d = (head.q * a * phi.tan()).min(b);
            Section { a, z_side: c + b - d, d, bb: 2.0 * b - d, z_tip: c + b, front: head.front }
        }
    })
}

/// 回転後の座標。
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Rotated {
    pub x: f64,
    pub y: f64,
    pub z: f64,
}

/// 共通の回転式（第3節）。theta はラジアン。
pub fn rotate(x: f64, y: f64, z: f64, theta: f64) -> Rotated {
    let (s, c) = theta.sin_cos();
    Rotated {
        x: x * c + z * s,
        y,
        z: -x * s + z * c,
    }
}

/// 高さ y の画面上の左右端と表面中心線（第6.1節・第6.3節）。
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Bounds {
    pub left: f64,
    pub right: f64,
    pub center: f64,
    /// 輪郭の幅の半分（楕円では第6.1節の R）。
    pub r: f64,
    /// 中心線から左輪郭までの距離 M_L
    pub margin_left: f64,
    /// 中心線から右輪郭までの距離 M_R
    pub margin_right: f64,
}

pub fn bounds(y: f64, theta: f64, head: Head) -> Result<Bounds> {
    let sec = section(y, head)?;
    let (s, cc) = theta.sin_cos();
    let xl = support(&sec, -1.0, s, cc);
    let xr = support(&sec, 1.0, s, cc);
    let left = xl.0 * cc + xl.1 * s;
    let right = xr.0 * cc + xr.1 * s;
    let center = sec.z_tip * s;
    Ok(Bounds {
        left,
        right,
        center,
        r: (right - left) / 2.0,
        margin_left: center - left,
        margin_right: right - center,
    })
}

/// 断面のうち、回転後の X を sigma の向きに最大にする点 (x, z)。
/// 凸な断面なので、これが今の輪郭の点になる（第6.2節の一般化）。
fn support(sec: &Section, sigma: f64, s: f64, cc: f64) -> (f64, f64) {
    let Section { a, z_side, bb, z_tip, front, .. } = *sec;
    // 楕円（中心 z_side、半径 a, rz）の sigma 方向の端。半径が 0 なら None。
    let ellipse_extreme = |rz: f64| -> Option<(f64, f64)> {
        let r = (a * cc).hypot(rz * s);
        (r >= EPS).then(|| (sigma * a * a * cc / r, z_side + sigma * rz * rz * s / r))
    };
    match front {
        Front::Ellipse => ellipse_extreme(sec.d).unwrap_or((0.0, z_side)),
        Front::Wedge { .. } => {
            let score = |p: &(f64, f64)| sigma * (p.0 * cc + p.1 * s);
            let mut best = (0.0, z_tip);
            let mut cands = vec![(a, z_side), (-a, z_side)];
            // 後ろ半分の楕円の端が、後ろ半分（z ≤ z_side）にあるときだけ候補にする。
            if sigma * s <= 0.0 {
                cands.extend(ellipse_extreme(bb));
            }
            for p in cands {
                if score(&p) > score(&best) + 1e-12 {
                    best = p;
                }
            }
            best
        }
    }
}

/// 局所座標 (x, y, z)。
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Local {
    pub x: f64,
    pub y: f64,
    pub z: f64,
}

/// 今、輪郭になっている表面の点（局所座標）。sigma = −1 が左端、+1 が右端。第6.2節。
pub fn contour_point(y: f64, sigma: f64, theta: f64, head: Head) -> Result<Local> {
    let sec = section(y, head)?;
    let (s, cc) = theta.sin_cos();
    let (x, z) = support(&sec, sigma, s, cc);
    Ok(Local { x, y, z })
}

/// 前面に貼った点の回転結果と可視判定（第7.1節・第8.1節）。
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SurfacePoint {
    pub local: Local,
    pub rotated: Rotated,
    pub facing: f64,
    pub visible: bool,
}

pub fn surface_point(x: f64, y: f64, theta: f64, head: Head) -> Result<SurfacePoint> {
    let sec = section(y, head)?;
    let Section { a, z_side, d, .. } = sec;
    let degenerate = match sec.front {
        Front::Ellipse => d <= EPS,
        Front::Wedge { .. } => false,
    };
    if !x.is_finite() || a <= EPS || degenerate || x.abs() > a + EPS {
        return Err(ModelError::EyeOffSurface);
    }
    let (s, cc) = theta.sin_cos();
    let (z, facing) = match sec.front {
        Front::Ellipse => {
            // 第7.1節・第8.1節
            let alpha = (x / a).clamp(-1.0, 1.0).asin();
            let (sa, ca) = alpha.sin_cos();
            (z_side + d * ca, ca * cc / d - sa * s / a)
        }
        Front::Wedge { .. } => {
            // V字の片側の平面。外向き法線は (sgn(x)·d, a) / √(a² + d²)。
            let u = (x.abs() / a).min(1.0);
            let n = a.hypot(d);
            let f = |sg: f64| (a * cc - sg * d * s) / n;
            let facing = if x > 0.0 {
                f(1.0)
            } else if x < 0.0 {
                f(-1.0)
            } else {
                f(1.0).max(f(-1.0))
            };
            (z_side + d * (1.0 - u), facing)
        }
    };
    Ok(SurfacePoint {
        local: Local { x, y, z },
        rotated: rotate(x, y, z, theta),
        facing,
        visible: facing >= -EPS,
    })
}

pub type Polyline = Vec<(f64, f64)>;

/// 可視判定が `want` と一致する点が連続する区間ごとに、`proj` で2次元化した線を返す。
/// 1点以下の区間は捨てる（第8.2節）。
pub fn runs_where(
    points: &[SurfacePoint],
    want: bool,
    proj: impl Fn(&SurfacePoint) -> (f64, f64),
) -> Vec<Polyline> {
    let mut runs = Vec::new();
    let mut run: Polyline = Vec::new();
    for p in points.iter().map(Some).chain(std::iter::once(None)) {
        match p {
            Some(p) if p.visible == want => run.push(proj(p)),
            _ => {
                if run.len() > 1 {
                    runs.push(std::mem::take(&mut run));
                } else {
                    run.clear();
                }
            }
        }
    }
    runs
}

/// 正面画像上の (X, Y) で可視区間を返す。
pub fn visible_runs(points: &[SurfacePoint]) -> Vec<Polyline> {
    runs_where(points, true, |p| (p.rotated.x, p.rotated.y))
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EyeRole {
    /// θ = 0 の画面左の目
    Left,
    /// θ = 0 の画面右の目
    Right,
    Near,
    Far,
}

impl EyeRole {
    pub fn label(self) -> &'static str {
        match self {
            Self::Left => "画面左",
            Self::Right => "画面右",
            Self::Near => "手前",
            Self::Far => "奥",
        }
    }
    /// 色分け用：手前／画面左 を一方、奥／画面右 をもう一方にする。
    pub fn is_near_style(self) -> bool {
        matches!(self, Self::Near | Self::Left)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Eye {
    /// 正面で画面左の目が −1、画面右の目が +1。
    pub side: f64,
    pub role: EyeRole,
    /// まぶた・瞳の可視区間（正面画像の X, Y）。
    pub runs: Vec<Polyline>,
    /// 目の高さ EYE_Y の目の横線のうち、見える部分の投影幅（モデル単位）。
    pub width: f64,
    /// 見える部分の投影範囲 (min X, max X)。見えない場合 None。
    pub span: Option<(f64, f64)>,
    /// 正面比 [%]。
    pub percent: f64,
    /// 目の高さ EYE_Y の目の横線（断面図用）。全サンプル点。
    pub line: Vec<SurfacePoint>,
}

/// 正面での片目の横幅（仕様の 30 × EYE_SCALE）。目幅の正面比の分母。
pub const EYE_WIDTH: f64 = 30.0 * EYE_SCALE;
/// まぶたの片側の縦幅（仕様の 8 × EYE_SCALE）。
pub const EYE_LID: f64 = 8.0 * EYE_SCALE;
/// 瞳の簡略曲線の半径（仕様の 3.2 × 5.6 に EYE_SCALE を掛けたもの）。
pub const PUPIL_RX: f64 = 3.2 * EYE_SCALE;
pub const PUPIL_RY: f64 = 5.6 * EYE_SCALE;

/// 片目を計算する。`center_x` は瞳（目の中心）の |x|。目頭は center_x − EYE_WIDTH / 2。
pub fn eye(side: f64, theta: f64, head: Head, center_x: f64) -> Result<Eye> {
    let inner = center_x - EYE_WIDTH / 2.0;
    let mut runs = Vec::new();
    for vertical in [-1.0, 1.0] {
        let pts = (0..=160)
            .map(|i| {
                let t = i as f64 / 160.0;
                surface_point(
                    side * (inner + EYE_WIDTH * t),
                    EYE_Y + vertical * EYE_LID * (std::f64::consts::PI * t).sin(),
                    theta,
                    head,
                )
            })
            .collect::<Result<Vec<_>>>()?;
        runs.extend(visible_runs(&pts));
    }
    let pupil = (0..=100)
        .map(|i| {
            let beta = i as f64 / 100.0 * std::f64::consts::TAU;
            surface_point(
                center_x * side + PUPIL_RX * beta.cos(),
                EYE_Y + PUPIL_RY * beta.sin(),
                theta,
                head,
            )
        })
        .collect::<Result<Vec<_>>>()?;
    runs.extend(visible_runs(&pupil));

    let line = (0..=300)
        .map(|i| surface_point(side * (inner + EYE_WIDTH * i as f64 / 300.0), EYE_Y, theta, head))
        .collect::<Result<Vec<_>>>()?;
    let span = line
        .iter()
        .filter(|p| p.visible)
        .map(|p| p.rotated.x)
        .fold(None, |acc: Option<(f64, f64)>, x| match acc {
            None => Some((x, x)),
            Some((lo, hi)) => Some((lo.min(x), hi.max(x))),
        });
    let width = span.map_or(0.0, |(lo, hi)| hi - lo);
    Ok(Eye {
        side,
        role: EyeRole::Left, // build_scene で上書き
        runs,
        width,
        span,
        percent: 100.0 * width / EYE_WIDTH,
        line,
    })
}

/// 上から見た目の高さ EYE_Y の断面図用データ（第9.2節）。座標は回転後の (X, Z)。
#[derive(Debug, Clone, PartialEq)]
pub struct CrossSection {
    /// 回転後の断面（閉じた線）。
    pub outline: Polyline,
    /// 正面（θ = 0, q = 1）の断面。参考線用。
    pub reference: Polyline,
    /// ● 今の輪郭の点 [左, 右]。
    pub contour_now: [(f64, f64); 2],
    /// □ 正面で輪郭だった点 (∓a, z_side) を回転したもの [左, 右]。
    pub contour_front: [(f64, f64); 2],
    /// 顔表面の中心線の点 (0, z_tip) を回転したもの。
    pub center: (f64, f64),
    pub bounds: Bounds,
}

/// 断面の外形を局所座標 (x, z) の閉じた線として返す。
pub fn section_outline(sec: &Section) -> Polyline {
    let Section { a, z_side, d, bb, z_tip, front } = *sec;
    let tau = std::f64::consts::TAU;
    match front {
        Front::Ellipse => (0..=180)
            .map(|i| {
                let al = i as f64 / 180.0 * tau;
                (a * al.sin(), z_side + d * al.cos())
            })
            .collect(),
        Front::Wedge { .. } => {
            // 右の角 → 後ろ → 左の角（後ろ半分の楕円）→ 前の先端 → 右の角
            let mut pts: Polyline = (0..=90)
                .map(|i| {
                    let al = std::f64::consts::FRAC_PI_2 + i as f64 / 90.0 * std::f64::consts::PI;
                    (a * al.sin(), z_side + bb * al.cos())
                })
                .collect();
            pts.push((0.0, z_tip));
            pts.push((a, z_side));
            pts
        }
    }
}

pub fn cross_section(theta: f64, head: Head) -> Result<CrossSection> {
    let sec = section(EYE_Y, head)?;
    let ref_sec = section(EYE_Y, Head { q: 1.0, ..head })?;
    let rot = |(x, z): (f64, f64), th: f64| {
        let r = rotate(x, EYE_Y, z, th);
        (r.x, r.z)
    };
    let cl = contour_point(EYE_Y, -1.0, theta, head)?;
    let cr = contour_point(EYE_Y, 1.0, theta, head)?;
    Ok(CrossSection {
        outline: section_outline(&sec).into_iter().map(|p| rot(p, theta)).collect(),
        reference: section_outline(&ref_sec),
        contour_now: [rot((cl.x, cl.z), theta), rot((cr.x, cr.z), theta)],
        contour_front: [rot((-sec.a, sec.z_side), theta), rot((sec.a, sec.z_side), theta)],
        center: rot((0.0, sec.z_tip), theta),
        bounds: bounds(EYE_Y, theta, head)?,
    })
}

#[derive(Debug, Clone, PartialEq)]
pub struct Scene {
    pub degrees: f64,
    pub q: f64,
    pub head: Head,
    /// 瞳（目の中心）の |x|。
    pub eye_x: f64,
    pub theta: f64,
    /// 回転後の輪郭。|θ| > 45° では断面モデルの輪郭を手描きの横顔へ寄せたもの。
    /// あごの裏から始まり、顎先・顔の前面・頭頂・後頭部を通ってうなじで終わる開いた線。
    /// |θ| ≤ 45° では始点と終点が一致し、真横では離れて首の開口になる。
    pub outline: Polyline,
    /// 正面の輪郭（q = 1, θ = 0）。回転角・q で動かない。`outline` と同じ順の点列。
    pub reference: Polyline,
    pub centerline: Polyline,
    /// 鼻の線。顔の上のレイヤーに描く。
    pub nose: Polyline,
    /// 鼻の下の顔を隠す領域（閉じた線）。
    pub nose_mask: Polyline,
    /// [画面左の目 (s=−1), 画面右の目 (s=+1)]
    pub eyes: [Eye; 2],
    pub chin: (f64, f64),
    pub eye_level: [(f64, f64); 2],
    pub section: CrossSection,
}

/// 鼻の線（正面画像の X, Y）。
fn nose_line(morph: &ProfileMorph) -> Result<Polyline> {
    NOSE_PROFILE.iter().map(|&[z, y]| morph.nose_point(y, z)).collect()
}

/// 鼻の下の顔を隠す領域（閉じた線）。鼻の線と同じ x = 0 の面にあるので、
/// 正面では幅がなくなり、真横では鼻の線と顔の線の奥までを覆う。
fn nose_mask(morph: &ProfileMorph) -> Result<Polyline> {
    let (top, bottom) = (NOSE_PROFILE[0][1], NOSE_PROFILE[NOSE_PROFILE.len() - 1][1]);
    let steps = ((bottom - top) * 2.0).ceil() as usize;
    let mut points = nose_line(morph)?;
    for i in (0..=steps).rev() {
        let y = top + (bottom - top) * i as f64 / steps as f64;
        // 両端では鼻の線に閉じる。
        let depth = NOSE_MASK_DEPTH * ((y - top).min(bottom - y) / 2.0).min(1.0);
        points.push(morph.nose_point(y, profile_at(&PROFILE_FRONT, y).0 - depth)?);
    }
    Ok(points)
}

/// 輪郭の点を描くときの補正。断面モデルの値を、真横で手描きの線に一致するようずらす。
/// `bounds` や断面図は断面モデルのままで、ここを通るのは顔の図の線だけ。
#[derive(Debug, Clone, Copy)]
struct ProfileMorph {
    theta: f64,
    head: Head,
    weight: f64,
    /// 頭の上部を正面の指定へ寄せる重み。
    frontal: f64,
    /// 真横で顔の前面が画面右なら +1、左なら −1。
    dir: f64,
}

impl ProfileMorph {
    fn new(degrees: f64, head: Head) -> Self {
        Self {
            theta: rad(degrees),
            head,
            weight: profile_weight(degrees),
            frontal: frontal_weight(degrees),
            dir: if degrees < 0.0 { -1.0 } else { 1.0 },
        }
    }

    /// 断面モデルの点 (`x`, `row`) を、真横のモデルの点 `side_x`（θ = +90°）が
    /// 手描きの点 `target` に重なるのと同じだけ動かす。
    /// `outward` は頭の上部の幅を広げる向き（画面右の輪郭で +1、左で −1、中心線で 0）。
    fn apply(
        &self,
        x: f64,
        row: f64,
        side_x: f64,
        target: (f64, f64),
        outward: f64,
    ) -> Result<(f64, f64)> {
        let (tx, ty) = target;
        let y = row - top_lift(row);
        let (dx, dy) = self.frontal_shift(row, y)?;
        let y = y + dy;
        Ok((
            x + outward * dx + self.weight * self.dir * (self.head.q * tx - side_x),
            y + self.weight * (ty - y),
        ))
    }

    /// 頭の上部を正面の指定へ寄せる量（半幅の増分、高さの増分）。
    fn frontal_shift(&self, row: f64, y: f64) -> Result<(f64, f64)> {
        if self.frontal <= 0.0 || row >= FRONTAL_TOP[FRONTAL_TOP.len() - 1][0] {
            return Ok((0.0, 0.0));
        }
        let (half, top_y) = profile_at(&FRONTAL_TOP, row);
        Ok((self.frontal * (half - shape(row, 1.0)?.a), self.frontal * (top_y - y)))
    }

    /// 顔の前面の輪郭の点。
    fn front(&self, row: f64) -> Result<(f64, f64)> {
        let b = bounds(row, self.theta, self.head)?;
        let x = if self.dir > 0.0 { b.right } else { b.left };
        let target = profile_at(&PROFILE_FRONT, row);
        self.apply(x, row, self.side(row)?.right, target, self.dir)
    }

    /// 後ろ側の輪郭の点。`table` は `PROFILE_BACK` か `PROFILE_JAW`。
    fn back(&self, row: f64, table: &[[f64; 3]]) -> Result<(f64, f64)> {
        let b = bounds(row, self.theta, self.head)?;
        let x = if self.dir > 0.0 { b.left } else { b.right };
        self.apply(x, row, self.side(row)?.left, profile_at(table, row), -self.dir)
    }

    /// 表面の中心線の点。真横では顔の前面の輪郭に重なる。
    fn center(&self, row: f64) -> Result<(f64, f64)> {
        let x = bounds(row, self.theta, self.head)?.center;
        self.apply(x, row, self.side(row)?.right, profile_at(&PROFILE_FRONT, row), 0.0)
    }

    /// 顔の中心の面 (x = 0) にある、高さ y・奥行き z（q = 1 の値）の鼻の点。
    /// 中心線からの張り出しが z と顔の線の差の sinθ 倍になるので、顔の線と同じ z の点は
    /// どの角度でも中心線の上にあり、鼻の両端が顔から離れない。
    fn nose_point(&self, y: f64, z: f64) -> Result<(f64, f64)> {
        let (center, _) = self.center(y)?;
        let face_z = profile_at(&PROFILE_FRONT, y).0;
        Ok((center + self.head.q * (z - face_z) * self.theta.sin(), y))
    }

    fn side(&self, row: f64) -> Result<Bounds> {
        bounds(row, std::f64::consts::FRAC_PI_2, self.head)
    }
}

/// 仕様どおりの楕円断面・目の間隔の初期値（EYE_CENTER_X）で場面を作る。
pub fn build_scene(degrees: f64, q: f64) -> Result<Scene> {
    build_scene_with(degrees, Head::ellipse(q), EYE_CENTER_X)
}

/// `eye_x` は瞳（目の中心）の |x|。EYE_CENTER_X_MIN〜EYE_CENTER_X_MAX。
pub fn build_scene_with(degrees: f64, head: Head, eye_x: f64) -> Result<Scene> {
    let q = head.q;
    check_view(degrees, q)?;
    check_head(head)?;
    if !eye_x.is_finite() || !(EYE_CENTER_X_MIN..=EYE_CENTER_X_MAX).contains(&eye_x) {
        return Err(ModelError::InvalidEyeLayout);
    }
    let theta = rad(degrees);
    let morph = ProfileMorph::new(degrees, head);
    let frontal = ProfileMorph::new(0.0, Head::ellipse(1.0));
    let split = (PROFILE_SPLIT_ROW * 2.0) as usize;
    let mut outline = Vec::with_capacity(692);
    let mut reference = Vec::with_capacity(692);
    // 後ろ側の下部（あごの裏）→ 顎先 → 前面 → 頭頂 → 後ろ側の上部 → うなじ。
    // 正面の輪郭 reference は θ > 0 と同じ向きに、画面左の下部から回る。
    for i in split..=344 {
        let y = i as f64 / 2.0;
        outline.push(morph.back(y, &PROFILE_JAW)?);
        reference.push(frontal.back(y, &PROFILE_JAW)?);
    }
    for i in (0..=344).rev() {
        let y = i as f64 / 2.0;
        outline.push(morph.front(y)?);
        reference.push(frontal.front(y)?);
    }
    for i in 0..=split {
        let y = i as f64 / 2.0;
        outline.push(morph.back(y, &PROFILE_BACK)?);
        reference.push(frontal.back(y, &PROFILE_BACK)?);
    }
    let centerline = (0..=172)
        .map(|y| morph.center(y as f64))
        .collect::<Result<Vec<_>>>()?;
    let mut eyes = [eye(-1.0, theta, head, eye_x)?, eye(1.0, theta, head, eye_x)?];
    for e in &mut eyes {
        e.role = if degrees == 0.0 {
            if e.side < 0.0 {
                EyeRole::Left
            } else {
                EyeRole::Right
            }
        } else if e.side * degrees.signum() < 0.0 {
            EyeRole::Near
        } else {
            EyeRole::Far
        };
    }
    let level_front = morph.front(EYE_Y)?;
    let level_back = morph.back(EYE_Y, &PROFILE_BACK)?;
    let eye_level = if morph.dir > 0.0 {
        [level_back, level_front]
    } else {
        [level_front, level_back]
    };
    Ok(Scene {
        degrees,
        q,
        head,
        eye_x,
        theta,
        outline,
        reference,
        centerline,
        nose: nose_line(&morph)?,
        nose_mask: nose_mask(&morph)?,
        eyes,
        chin: (60.0 * q * theta.sin(), Y_MAX),
        eye_level,
        section: cross_section(theta, head)?,
    })
}

/// 独立した目の平面モデル（第4節）。角度は度。
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PlanarEye {
    pub d: f64,
    pub near: f64,
    pub far: f64,
}

pub fn planar_eye_widths(degrees: f64, phi_degrees: f64, w0: f64) -> Result<PlanarEye> {
    check_view(degrees, 1.0)?;
    if !phi_degrees.is_finite()
        || !(0.0..90.0).contains(&phi_degrees)
        || !w0.is_finite()
        || w0 <= 0.0
    {
        return Err(ModelError::InvalidPlanarInput);
    }
    let psi = rad(degrees.abs());
    let d = w0 * rad(phi_degrees).tan();
    let near = w0 * psi.cos() + d * psi.sin();
    let far = w0 * psi.cos() - d * psi.sin();
    Ok(PlanarEye {
        d,
        near: near.max(0.0),
        far: far.max(0.0),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    const TOL: f64 = 1e-8;
    const QS: [f64; 3] = [0.55, 1.0, 1.35];

    /// 形によらず成り立つ条件を確かめる頭の形（楕円・V字の初期値・平ら・最大の傾き）。
    fn heads(q: f64) -> [Head; 4] {
        [
            Head::ellipse(q),
            Head::wedge(q, WEDGE_PHI_DEFAULT),
            Head::wedge(q, WEDGE_PHI_MIN),
            Head::wedge(q, WEDGE_PHI_MAX),
        ]
    }

    fn scene(deg: f64, head: Head) -> Scene {
        build_scene_with(deg, head, EYE_CENTER_X).unwrap()
    }

    fn near_far(s: &Scene) -> (&Eye, &Eye) {
        let near = s.eyes.iter().find(|e| e.role == EyeRole::Near).unwrap();
        let far = s.eyes.iter().find(|e| e.role == EyeRole::Far).unwrap();
        (near, far)
    }

    #[test]
    fn shape_hits_table_values() {
        for r in A_ROWS {
            assert!((shape(r[0], 1.0).unwrap().a - r[1]).abs() < TOL);
        }
        for r in BC_ROWS {
            let s = shape(r[0], 1.0).unwrap();
            assert!((s.b - r[1]).abs() < TOL);
            assert!((s.c - r[2]).abs() < TOL);
        }
        let s = shape(100.0, 1.3).unwrap();
        assert!((s.b - 65.05 * 1.3).abs() < TOL && (s.c - 2.08 * 1.3).abs() < TOL);
    }

    #[test]
    fn wedge_keeps_centerline_and_back() {
        for q in QS {
            for i in 0..=344 {
                let y = i as f64 / 2.0;
                let sh = shape(y, q).unwrap();
                let s = section(y, Head::wedge(q, WEDGE_PHI_DEFAULT)).unwrap();
                assert!((s.z_tip - (sh.c + sh.b)).abs() < TOL);
                assert!((s.z_side - s.bb - (sh.c - sh.b)).abs() < TOL);
                assert!(s.d >= 0.0 && s.bb >= s.d - TOL);
            }
        }
    }

    #[test]
    fn frontal_view_is_identity() {
        for q in QS {
            for head in heads(q) {
                let s = scene(0.0, head);
                assert!(s.chin.0.abs() < TOL);
                for e in &s.eyes {
                    assert_eq!(e.percent.round(), 100.0);
                    for p in &e.line {
                        assert!((p.rotated.x - p.local.x).abs() < TOL);
                    }
                }
                assert_eq!(s.eyes[0].role, EyeRole::Left);
                assert_eq!(s.eyes[1].role, EyeRole::Right);
                // 正面では q や断面の形によらず、輪郭は ±a、中心線は 0。
                for (p, r) in s.outline.iter().zip(&s.reference) {
                    assert!((p.0 - r.0).abs() < TOL);
                }
                assert!(s.centerline.iter().all(|p| p.0.abs() < TOL));
            }
        }
    }

    #[test]
    fn outline_is_mirror_symmetric() {
        for q in QS {
            for head in heads(q) {
                for deg in [1.0, 17.0, 30.0, 60.0, 89.0, 90.0] {
                    for i in 0..=344 {
                        let y = i as f64 / 2.0;
                        let p = bounds(y, rad(deg), head).unwrap();
                        let m = bounds(y, rad(-deg), head).unwrap();
                        assert!((m.left + p.right).abs() < TOL);
                        assert!((m.right + p.left).abs() < TOL);
                    }
                }
            }
        }
    }

    #[test]
    fn chin_offset_matches_formula() {
        for q in QS {
            for head in heads(q) {
                for deg in [-90.0, -45.0, 0.0, 30.0, 90.0] {
                    let s = scene(deg, head);
                    let want = 60.0 * q * rad(deg).sin();
                    assert!((s.chin.0 - want).abs() < TOL);
                    let b = bounds(172.0, rad(deg), head).unwrap();
                    assert!((b.left - want).abs() < TOL && (b.right - want).abs() < TOL);
                }
            }
        }
    }

    #[test]
    fn far_eye_vanishes_at_90() {
        for q in QS {
            for head in heads(q) {
                for deg in [90.0, -90.0] {
                    let s = scene(deg, head);
                    let (_, far) = near_far(&s);
                    // 平らな前面（φ = 0）では真横でちょうど面が縦一直線になり（F = 0 の境界）、
                    // 幅 0 の線として残る。それ以外は裏側に回って見えなくなる。
                    assert!(far.width.abs() < 1e-9);
                }
            }
        }
    }

    /// ●の点を回転した X が輪郭の左右端に一致し、断面上のどの点よりも外側にある。
    #[test]
    fn contour_point_is_the_extreme_point() {
        for q in QS {
            for head in heads(q) {
                for deg in [-90.0, -50.0, -30.0, 0.0, 10.0, 30.0, 75.0, 90.0] {
                    let th = rad(deg);
                    for y in [0.0, 1.0, 50.0, EYE_Y, 120.0, 150.0, 172.0] {
                        let b = bounds(y, th, head).unwrap();
                        let l = contour_point(y, -1.0, th, head).unwrap();
                        let r = contour_point(y, 1.0, th, head).unwrap();
                        assert!((rotate(l.x, l.y, l.z, th).x - b.left).abs() < TOL);
                        assert!((rotate(r.x, r.y, r.z, th).x - b.right).abs() < TOL);
                        let sec = section(y, head).unwrap();
                        for (x, z) in section_outline(&sec) {
                            let xx = rotate(x, y, z, th).x;
                            assert!(xx >= b.left - 1e-6 && xx <= b.right + 1e-6);
                        }
                    }
                }
            }
        }
    }

    /// 楕円では第6.1節の厳密な式 X = cS ± R と一致する。
    #[test]
    fn ellipse_bounds_match_spec_formula() {
        for q in QS {
            for deg in [-80.0, -30.0, 0.0, 45.0, 90.0] {
                let th = rad(deg);
                let (s, c) = th.sin_cos();
                for y in [0.0, 30.0, EYE_Y, 140.0, 172.0] {
                    let sh = shape(y, q).unwrap();
                    let r = (sh.a * c).hypot(sh.b * s);
                    let b = bounds(y, th, Head::ellipse(q)).unwrap();
                    assert!((b.left - (sh.c * s - r)).abs() < TOL);
                    assert!((b.right - (sh.c * s + r)).abs() < TOL);
                }
            }
        }
    }

    /// V字では目が平面上にあるので、第4節の平面モデルの式どおりに変化する。
    /// 傾きの tan は d / a（q を含む）。奥の目は 90° − atan(d/a) を超えると一度に消える。
    #[test]
    fn wedge_eye_follows_planar_model() {
        for q in QS {
            let head = Head::wedge(q, WEDGE_PHI_DEFAULT);
            let sec = section(EYE_Y, head).unwrap();
            let t = sec.d / sec.a;
            let vanish = 90.0 - t.atan().to_degrees();
            for deg in [5.0, 20.0, 45.0, 60.0, 80.0] {
                let s = scene(deg, head);
                let (near, far) = near_far(&s);
                let th = rad(deg);
                let want_near = 100.0 * (th.cos() + t * th.sin());
                assert!((near.percent - want_near).abs() < 1e-6, "q={q} deg={deg}");
                if deg < vanish - 0.5 {
                    let want_far = 100.0 * (th.cos() - t * th.sin());
                    assert!((far.percent - want_far).abs() < 1e-6, "q={q} deg={deg}");
                } else if deg > vanish + 0.5 {
                    assert_eq!(far.width, 0.0);
                }
            }
        }
    }

    #[test]
    fn all_values_finite_and_nonnegative() {
        for q in QS {
            for head in heads(q) {
                for deg in (-90..=90).step_by(3) {
                    let s = scene(deg as f64, head);
                    for e in &s.eyes {
                        assert!(e.width.is_finite() && e.width >= 0.0);
                        assert!(e.percent.is_finite());
                        for run in &e.runs {
                            assert!(run.len() > 1);
                            assert!(run.iter().all(|p| p.0.is_finite() && p.1.is_finite()));
                        }
                    }
                    assert!(s.outline.iter().all(|p| p.0.is_finite()));
                    assert!(s.section.outline.iter().all(|p| p.0.is_finite() && p.1.is_finite()));
                }
            }
        }
    }

    #[test]
    fn far_eye_never_reappears() {
        for q in QS {
            for head in heads(q) {
                let mut prev = f64::INFINITY;
                for deg in 0..=90 {
                    let far = &scene(deg as f64, head).eyes[1];
                    assert!(far.width <= prev + 1e-9, "q={q} deg={deg}");
                    prev = far.width;
                }
            }
        }
    }

    #[test]
    fn planar_model_phi_zero() {
        for deg in [-60.0, 0.0, 30.0, 90.0] {
            let p = planar_eye_widths(deg, 0.0, 40.0).unwrap();
            let want = (40.0 * rad(deg).cos()).max(0.0);
            assert!((p.near - want).abs() < TOL && (p.far - want).abs() < TOL);
        }
        // 消失角 ψ = 90° − φ
        let p = planar_eye_widths(70.0, 20.0, 40.0).unwrap();
        assert!(p.far.abs() < 1e-9);
    }

    /// 仕様 第13.2節の表の形式で、楕円断面での値を固定する。
    /// 輪郭 a₀・前後の厚み b₀, c₀・目の配置と大きさを参考画像に合わせたため、目幅は仕様書の
    /// 数値（30°で 118%／55% → 124%／49% など。目の間隔は初期値 EYE_CENTER_X）とは異なる。
    /// 顎先の横ずれは c(172) = 60q が変わらないので仕様書と同じ。
    #[test]
    fn eye_width_table_for_current_shape() {
        // (|θ|, 手前%, 奥%, 顎先の横ずれ)
        let table = [
            (0.0, 100.0, 100.0, 0.0),
            (30.0, 124.0, 49.0, 30.0),
            (60.0, 116.0, 5.0, 51.9615),
            (90.0, 76.0, 0.0, 60.0),
        ];
        for (deg, near_pct, far_pct, chin) in table {
            for sign in [1.0, -1.0] {
                let s = build_scene(sign * deg, 1.0).unwrap();
                let (n, f) = if deg == 0.0 {
                    (&s.eyes[0], &s.eyes[1])
                } else {
                    near_far(&s)
                };
                assert_eq!(n.percent.round(), near_pct, "near {deg}");
                assert_eq!(f.percent.round(), far_pct, "far {deg}");
                assert!((s.chin.0.abs() - chin).abs() < 1e-4);
            }
        }
    }

    #[test]
    fn eye_spacing_range_is_valid_everywhere() {
        for eye_x in [EYE_CENTER_X_MIN, 28.0, EYE_CENTER_X, EYE_CENTER_X_MAX] {
            for q in QS {
                for head in heads(q) {
                    for deg in (-90..=90).step_by(5) {
                        let s = build_scene_with(deg as f64, head, eye_x).unwrap();
                        for e in &s.eyes {
                            assert!(e.width.is_finite() && e.width >= 0.0);
                        }
                    }
                    let s = build_scene_with(0.0, head, eye_x).unwrap();
                    for e in &s.eyes {
                        assert_eq!(e.percent.round(), 100.0);
                        let (lo, hi) = e.span.unwrap();
                        assert!(((lo + hi) / 2.0 - e.side * eye_x).abs() < 1e-9);
                    }
                }
            }
        }
    }

    /// |θ| ≤ 45° の輪郭は、頭の上部を除いて断面モデルのままで、線は閉じている。
    #[test]
    fn profile_does_not_touch_views_up_to_45() {
        for q in QS {
            for head in heads(q) {
                for deg in [-45.0, -20.0, 0.0, 30.0, 45.0] {
                    let s = scene(deg, head);
                    assert_eq!(s.outline.first(), s.outline.last());
                    let from = if deg.abs() == 45.0 {
                        TOP_LIFT_RANGE
                    } else {
                        FRONTAL_TOP[FRONTAL_TOP.len() - 1][0]
                    };
                    for &(x, y) in s.outline.iter().filter(|p| p.1 >= from) {
                        let b = bounds(y, rad(deg), head).unwrap();
                        assert!(
                            (x - b.left).abs() < TOL || (x - b.right).abs() < TOL,
                            "q={q} deg={deg} y={y}"
                        );
                    }
                }
            }
        }
    }

    /// 正面の頭の上部は手描きの線に一致し、左右対称で、下の輪郭となめらかにつながる。
    #[test]
    fn frontal_top_matches_drawing() {
        for q in QS {
            for head in heads(q) {
                let s = scene(0.0, head);
                for (p, r) in s.outline.iter().zip(&s.reference) {
                    assert!((p.0 - r.0).abs() < TOL && (p.1 - r.1).abs() < TOL);
                }
                for r in FRONTAL_TOP {
                    for side in [-1.0, 1.0] {
                        let gap = s
                            .outline
                            .iter()
                            .map(|p| (p.0 - side * r[1]).hypot(p.1 - r[2]))
                            .fold(f64::INFINITY, f64::min);
                        assert!(gap < 1.0, "q={q} point={r:?} off by {gap}");
                    }
                }
                let end = FRONTAL_TOP[FRONTAL_TOP.len() - 1];
                assert!((end[1] - shape(end[0], 1.0).unwrap().a).abs() < 0.01);
            }
        }
        assert!((FRONTAL_TOP[0][2] - PROFILE_FRONT[0][2]).abs() < TOL);
    }

    /// 頭頂の高さはどの角度でも真横の指定と同じで、輪郭の高さの順番は入れ替わらない。
    #[test]
    fn head_top_height_is_constant() {
        let top = |s: &Scene| s.outline.iter().map(|p| p.1).fold(f64::INFINITY, f64::min);
        for q in QS {
            for head in heads(q) {
                for deg in (-90..=90).step_by(15) {
                    let s = scene(deg as f64, head);
                    assert!((top(&s) - PROFILE_FRONT[0][2]).abs() < TOL, "{deg}");
                    assert!((s.eye_level[0].1 - EYE_Y).abs() < 0.2);
                }
            }
        }
        for i in 0..80 {
            let (a, b) = (i as f64 / 2.0, (i + 1) as f64 / 2.0);
            assert!(a - top_lift(a) < b - top_lift(b));
        }
    }

    /// 真横の輪郭は手描きの線に一致する。断面の形によらず同じで、X は q 倍になる。
    #[test]
    fn profile_matches_drawing_at_90() {
        let all = PROFILE_BACK.iter().chain(&PROFILE_FRONT).chain(&PROFILE_JAW);
        for q in QS {
            for head in heads(q) {
                let s = scene(90.0, head);
                let m = scene(-90.0, head);
                for (p, n) in s.outline.iter().zip(&m.outline) {
                    assert!((p.0 + n.0).abs() < TOL && (p.1 - n.1).abs() < TOL);
                }
                for r in all.clone() {
                    let nearest = s
                        .outline
                        .iter()
                        .map(|p| (p.0 - q * r[1]).hypot(p.1 - r[2]))
                        .fold(f64::INFINITY, f64::min);
                    assert!(nearest < 1.2 * q.max(1.0), "q={q} point={r:?} off by {nearest}");
                }
                let top = s.outline.iter().map(|p| p.1).fold(f64::INFINITY, f64::min);
                assert!((top - PROFILE_FRONT[0][2]).abs() < TOL);
                let (a, b) = (s.outline[0], *s.outline.last().unwrap());
                assert!((a.0 - b.0).hypot(a.1 - b.1) > 10.0, "首の開口が閉じている");
                assert!((s.eye_level[0].0 - bounds(EYE_Y, rad(90.0), head).unwrap().left).abs() < 1.0);
            }
        }
    }

    /// 鼻は正面で縦の直線、真横で指定の形。どの角度でも両端は顔の中心線から離れない。
    #[test]
    fn nose_rotates_with_the_head() {
        for q in QS {
            for head in heads(q) {
                let front = scene(0.0, head);
                assert!(front.nose.iter().chain(&front.nose_mask).all(|p| p.0.abs() < TOL));
                let side = scene(90.0, head);
                for (p, r) in side.nose.iter().zip(&NOSE_PROFILE) {
                    assert!((p.0 - q * r[0]).abs() < 0.2 * q && p.1 == r[1]);
                }
                for deg in [20.0, 45.0, 60.0, 75.0, 90.0] {
                    let (s, m) = (scene(deg, head), scene(-deg, head));
                    for (p, n) in s.nose.iter().zip(&m.nose) {
                        assert!((p.0 + n.0).abs() < TOL);
                    }
                    let morph = ProfileMorph::new(deg, head);
                    for end in [s.nose[0], *s.nose.last().unwrap()] {
                        assert!((end.0 - morph.center(end.1).unwrap().0).abs() < 0.2 * q);
                    }
                }
                for end in [side.nose[0], *side.nose.last().unwrap()] {
                    let gap = side
                        .outline
                        .iter()
                        .map(|p| (p.0 - end.0).hypot(p.1 - end.1))
                        .fold(f64::INFINITY, f64::min);
                    assert!(gap < 0.5 * q.max(1.0), "q={q} {end:?} off by {gap}");
                }
            }
        }
    }

    /// 真横で、鼻の範囲の顔の線は隠す領域の中にある（鼻の線と奥の線のあいだ）。
    #[test]
    fn nose_mask_covers_the_face_line_at_90() {
        let (top, bottom) = (NOSE_PROFILE[0][1], NOSE_PROFILE[NOSE_PROFILE.len() - 1][1]);
        let mask = scene(90.0, Head::wedge(1.0, WEDGE_PHI_DEFAULT)).nose_mask;
        let inside = |(x, y): (f64, f64)| {
            let mut hit = false;
            for (a, b) in mask.iter().zip(mask.iter().cycle().skip(1)) {
                if (a.1 > y) != (b.1 > y) && x < a.0 + (y - a.1) / (b.1 - a.1) * (b.0 - a.0) {
                    hit = !hit;
                }
            }
            hit
        };
        for i in 1..100 {
            let y = top + 2.5 + (bottom - top - 5.0) * i as f64 / 100.0;
            assert!(inside(profile_at(&PROFILE_FRONT, y)), "y={y}");
        }
    }

    /// 45°〜90° のあいだ、線が飛ばずに動く（隣の角度との差が小さい）。
    #[test]
    fn profile_moves_smoothly() {
        let head = Head::wedge(1.0, WEDGE_PHI_DEFAULT);
        for deg in 45..90 {
            let (a, b) = (scene(deg as f64, head), scene(deg as f64 + 1.0, head));
            for (p, n) in a.outline.iter().zip(&b.outline) {
                assert!((p.0 - n.0).hypot(p.1 - n.1) < 8.0, "deg={deg}");
            }
        }
    }

    #[test]
    fn rejects_invalid_input() {
        let h = Head::ellipse(1.0);
        assert_eq!(build_scene(91.0, 1.0), Err(ModelError::InvalidView));
        assert_eq!(build_scene(f64::NAN, 1.0), Err(ModelError::InvalidView));
        assert_eq!(build_scene(0.0, 0.0), Err(ModelError::InvalidView));
        assert_eq!(build_scene_with(0.0, h, 21.9), Err(ModelError::InvalidEyeLayout));
        assert_eq!(build_scene_with(0.0, h, 44.1), Err(ModelError::InvalidEyeLayout));
        assert_eq!(build_scene_with(0.0, h, f64::NAN), Err(ModelError::InvalidEyeLayout));
        assert_eq!(
            build_scene_with(0.0, Head::wedge(1.0, 41.0), EYE_CENTER_X),
            Err(ModelError::InvalidHead)
        );
        assert_eq!(
            build_scene_with(0.0, Head::wedge(1.0, -1.0), EYE_CENTER_X),
            Err(ModelError::InvalidHead)
        );
        assert_eq!(shape(-1.0, 1.0), Err(ModelError::InvalidShapeInput));
        assert_eq!(shape(172.5, 1.0), Err(ModelError::InvalidShapeInput));
        for head in heads(1.0) {
            assert_eq!(surface_point(70.0, 88.0, 0.0, head), Err(ModelError::EyeOffSurface));
            assert_eq!(surface_point(0.0, 172.0, 0.0, head), Err(ModelError::EyeOffSurface));
        }
    }
}
