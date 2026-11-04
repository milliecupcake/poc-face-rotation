//! 目の画像素材を楕円表面に貼って回転させる（仕様 第7.1節・第8.1節の応用）。
//!
//! 画像は正面から見た絵として扱い、画像の px 座標 (u, v) を正面の顔座標 (x, y) へ
//! 線形に対応させる。その (x, y) を前面の楕円表面に貼り、同じ回転式で回す。
//! 画像は小さなセルに分け、各セルの3隅から求めたアフィン変換で描く。
//! 裏側を向いたセル（F < −10⁻¹⁰）は描かない。
//!
//! 画像に依存する値（サイズ・瞳の位置・各目の範囲）は `EyeTexture` にまとめてある。
//! 素材を差し替えるときはここを調整する。

use crate::model::{surface_point, Head, Result, EPS, EYE_SCALE, EYE_Y};

/// 目の画像素材の配置設定。座標の単位は画像の px。
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct EyeTexture {
    pub href: &'static str,
    pub width: f64,
    pub height: f64,
    /// 画面左の目の瞳（虹彩）中心。
    pub left_pupil: (f64, f64),
    /// 画面右の目の瞳（虹彩）中心。
    pub right_pupil: (f64, f64),
    /// 瞳の中心を置く高さ y。モデルの瞳の簡略曲線と同じ高さ。
    /// 左右の位置（|x|）は描画時に `eye_x` として渡す。
    pub model_pupil_y: f64,
    /// 画像 px 1 あたりのモデル単位。
    pub px_scale: f64,
    /// 画面左の目（まゆ毛を含む）の範囲 [u0, v0, u1, v1]。
    pub left_region: [f64; 4],
    /// 画面右の目（まゆ毛を含む）の範囲 [u0, v0, u1, v1]。
    pub right_region: [f64; 4],
    /// セルの分割数（横, 縦）。
    pub grid: (usize, usize),
}

/// 添付素材 `assets/eyes.png`（416×227、背景透過）用の設定。
pub const EYES: EyeTexture = EyeTexture {
    href: "assets/eyes.png",
    width: 416.0,
    height: 227.0,
    left_pupil: (102.5, 131.5),
    right_pupil: (311.5, 134.5),
    model_pupil_y: EYE_Y,
    // 仕様どおりの大きさ（瞳 ±28 に両瞳の間隔 209px を合わせた縮尺）に EYE_SCALE を掛ける。
    px_scale: 2.0 * 28.0 / 209.0 * EYE_SCALE,
    left_region: [4.0, 36.0, 166.0, 190.0],
    right_region: [241.0, 36.0, 412.0, 190.0],
    grid: (28, 11),
};

impl EyeTexture {
    /// 画像 px 1 あたりのモデル単位。
    pub fn scale(&self) -> f64 {
        self.px_scale
    }

    /// 画像 px (u, v) → 正面の顔座標 (x, y)。
    /// 各目の瞳の中心を (±eye_x, model_pupil_y) に合わせて縮尺 px_scale で置く。
    pub fn to_face(&self, u: f64, v: f64, eye_x: f64) -> (f64, f64) {
        let k = self.px_scale;
        let mid = (self.left_pupil.0 + self.right_pupil.0) / 2.0;
        let (pupil, side) = if u < mid {
            (self.left_pupil, -1.0)
        } else {
            (self.right_pupil, 1.0)
        };
        let cv = (self.left_pupil.1 + self.right_pupil.1) / 2.0;
        (
            side * eye_x + (u - pupil.0) * k,
            self.model_pupil_y + (v - cv) * k,
        )
    }
}

/// 描画する1セル。`matrix` は画像 px → 画面上のモデル座標 (X, Y) のアフィン変換
/// (a, b, c, d, e, f)：X = a·u + c·v + e、Y = b·u + d·v + f。
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Cell {
    pub matrix: [f64; 6],
    /// 切り出す画像の範囲 [u, v, w, h]（継ぎ目を隠すため少し広げてある）。
    pub src: [f64; 4],
    /// 画面左の目なら −1、画面右の目なら +1。
    pub side: f64,
}

/// 継ぎ目の隙間を防ぐため、各セルの切り出しを広げる量 [px]。
const OVERLAP: f64 = 0.6;

/// 回転角 theta（ラジアン）、頭の形 head、瞳の |x| である eye_x で、見えているセルを返す。
pub fn cells(tex: &EyeTexture, theta: f64, head: Head, eye_x: f64) -> Result<Vec<Cell>> {
    let (nu, nv) = tex.grid;
    let mut out = Vec::with_capacity(2 * nu * nv);
    for (side, r) in [(-1.0, tex.left_region), (1.0, tex.right_region)] {
        let du = (r[2] - r[0]) / nu as f64;
        let dv = (r[3] - r[1]) / nv as f64;
        for j in 0..nv {
            let v0 = r[1] + dv * j as f64;
            let v1 = v0 + dv;
            for i in 0..nu {
                let u0 = r[0] + du * i as f64;
                let u1 = u0 + du;
                if let Some(cell) = cell(tex, side, (u0, v0, u1, v1), theta, head, eye_x)? {
                    out.push(cell);
                }
            }
        }
    }
    Ok(out)
}

fn cell(
    tex: &EyeTexture,
    side: f64,
    (u0, v0, u1, v1): (f64, f64, f64, f64),
    theta: f64,
    head: Head,
    eye_x: f64,
) -> Result<Option<Cell>> {
    // 表面の外へはみ出す隅があるセルは描かない（素材の余白部分）。
    let project = |u: f64, v: f64| -> Result<Option<crate::model::SurfacePoint>> {
        let (x, y) = tex.to_face(u, v, eye_x);
        match surface_point(x, y, theta, head) {
            Ok(p) => Ok(Some(p)),
            Err(crate::model::ModelError::EyeOffSurface) => Ok(None),
            Err(e) => Err(e),
        }
    };
    let (Some(p00), Some(p10), Some(p01), Some(pc)) = (
        project(u0, v0)?,
        project(u1, v0)?,
        project(u0, v1)?,
        project((u0 + u1) / 2.0, (v0 + v1) / 2.0)?,
    ) else {
        return Ok(None);
    };
    if pc.facing < -EPS {
        return Ok(None);
    }
    let a = (p10.rotated.x - p00.rotated.x) / (u1 - u0);
    let c = (p01.rotated.x - p00.rotated.x) / (v1 - v0);
    let d = (p01.rotated.y - p00.rotated.y) / (v1 - v0);
    let e = p00.rotated.x - a * u0 - c * v0;
    let f = p00.rotated.y - d * v0;
    Ok(Some(Cell {
        matrix: [a, 0.0, c, d, e, f],
        src: [
            u0 - OVERLAP,
            v0 - OVERLAP,
            u1 - u0 + 2.0 * OVERLAP,
            v1 - v0 + 2.0 * OVERLAP,
        ],
        side,
    }))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::{rad, EYE_CENTER_X, EYE_CENTER_X_MAX, EYE_CENTER_X_MIN, WEDGE_PHI_DEFAULT};

    fn heads(q: f64) -> [Head; 3] {
        [Head::ellipse(q), Head::wedge(q, WEDGE_PHI_DEFAULT), Head::wedge(q, 0.0)]
    }

    #[test]
    fn pupils_land_on_model_pupils() {
        for eye_x in [EYE_CENTER_X_MIN, EYE_CENTER_X, EYE_CENTER_X_MAX] {
            let (x, y) = EYES.to_face(EYES.left_pupil.0, EYES.left_pupil.1, eye_x);
            let (x2, y2) = EYES.to_face(EYES.right_pupil.0, EYES.right_pupil.1, eye_x);
            assert!((x + eye_x).abs() < 1e-9 && (x2 - eye_x).abs() < 1e-9);
            assert!(((y + y2) / 2.0 - EYE_Y).abs() < 1e-9);
        }
    }

    #[test]
    fn frontal_view_is_plain_image_placement() {
        let k = EYES.scale();
        for head in heads(1.0) {
            let cells = cells(&EYES, 0.0, head, EYE_CENTER_X).unwrap();
            assert!(!cells.is_empty());
            for c in &cells {
                let [a, b, cc, d, _, _] = c.matrix;
                assert!((a - k).abs() < 1e-9 && b == 0.0 && cc.abs() < 1e-9 && (d - k).abs() < 1e-9);
            }
        }
    }

    #[test]
    fn far_eye_disappears_at_90() {
        for q in [0.55, 1.0, 1.35] {
            for head in heads(q) {
                // 奥の目のセルは描かないか、平らな前面（φ = 0）の境界で幅 0 になる。
                let zero = |c: &Cell| c.matrix[0].abs() < 1e-9;
                let right = cells(&EYES, rad(90.0), head, EYE_CENTER_X).unwrap();
                assert!(right.iter().all(|c| c.side < 0.0 || zero(c)));
                let left = cells(&EYES, rad(-90.0), head, EYE_CENTER_X).unwrap();
                assert!(left.iter().all(|c| c.side > 0.0 || zero(c)));
            }
        }
    }

    #[test]
    fn all_finite_over_full_range() {
        for eye_x in [EYE_CENTER_X_MIN, EYE_CENTER_X, EYE_CENTER_X_MAX] {
            for q in [0.55, 1.0, 1.35] {
                for head in heads(q) {
                    for deg in (-90..=90).step_by(3) {
                        for c in cells(&EYES, rad(deg as f64), head, eye_x).unwrap() {
                            assert!(c.matrix.iter().all(|v| v.is_finite()));
                        }
                    }
                }
            }
        }
    }
}
