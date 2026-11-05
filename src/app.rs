//! Leptos による UI と SVG 描画（仕様 第9節）。
//! 幾何計算は `model` に任せ、ここではモデル単位の座標を固定 viewBox の SVG に描くだけにする。

use leptos::prelude::*;

use crate::model::{
    build_scene_with, runs_where, Eye, Head, Polyline, Scene, EYE_CENTER_X, EYE_CENTER_X_MAX,
    EYE_CENTER_X_MIN, EYE_IMAGE_FIT_MAX, EYE_WIDTH, EYE_Y, PITCH_MAX, WEDGE_PHI_DEFAULT, WEDGE_PHI_MAX,
    WEDGE_PHI_MIN,
};
use crate::texture::{self, EYES};

/// 断面図で投影幅を示す基準線の Z（観察者側）。
const PROJ_Z: f64 = 104.0;

fn fmt(v: f64) -> String {
    let s = format!("{v:.2}");
    if s == "-0.00" {
        "0.00".into()
    } else {
        s
    }
}

fn path_d(points: &[(f64, f64)], closed: bool) -> String {
    let mut d = String::with_capacity(points.len() * 14);
    for (i, (x, y)) in points.iter().enumerate() {
        d.push(if i == 0 { 'M' } else { 'L' });
        d.push_str(&fmt(*x));
        d.push(',');
        d.push_str(&fmt(*y));
    }
    if closed {
        d.push('Z');
    }
    d
}

fn eye_class(e: &Eye) -> &'static str {
    if e.role.is_near_style() {
        "near"
    } else {
        "far"
    }
}

fn angle_label(deg: f64) -> String {
    if deg == 0.0 {
        "正面 0°".into()
    } else {
        format!("{}{}°", if deg < 0.0 { "左" } else { "右" }, deg.abs())
    }
}

fn pitch_label(deg: f64) -> String {
    if deg == 0.0 {
        "水平 0°".into()
    } else {
        format!("{}{}°", if deg < 0.0 { "下" } else { "上" }, deg.abs())
    }
}

/// 手前（または画面左）を先に並べる。
fn ordered_eyes(s: &Scene) -> [&Eye; 2] {
    if s.eyes[0].role.is_near_style() {
        [&s.eyes[0], &s.eyes[1]]
    } else {
        [&s.eyes[1], &s.eyes[0]]
    }
}

#[component]
pub fn App() -> impl IntoView {
    let (deg, set_deg) = signal(0.0_f64);
    let (pitch, set_pitch) = signal(0.0_f64);
    let (q, set_q) = signal(1.0_f64);
    let (show_ref, set_show_ref) = signal(false);
    let (use_image, set_use_image) = signal(true);
    let (eye_x, set_eye_x) = signal(EYE_CENTER_X);

    // 断面の前面の形：true で V字、false で楕円（仕様）。
    let (wedge, set_wedge) = signal(true);
    let (phi, set_phi) = signal(WEDGE_PHI_DEFAULT);

    let head = move || {
        let head = if wedge.get() {
            Head::wedge(q.get(), phi.get())
        } else {
            Head::ellipse(q.get())
        };
        head.with_pitch(pitch.get())
    };
    let scene = Memo::new(move |_| build_scene_with(deg.get(), head(), eye_x.get()));
    let eye_x_label = move || format!("瞳 ±{:.1}（間隔 {:.1}）", eye_x.get(), 2.0 * eye_x.get());

    view! {
        <main>
            <header>
                <h1>"アニメ顔の横回転モデル"</h1>
                <p class="lede">
                    "目と輪郭を同じ楕円断面モデルで回転させ、目の横幅・輪郭・中心線・顎先の位置を計算します。"
                </p>
            </header>

            <section class="controls" aria-label="操作">
                <label class="slider">
                    <span class="row">
                        <span>"横回転 θ"</span>
                        <output>{move || angle_label(deg.get())}</output>
                    </span>
                    <input
                        type="range" min="-90" max="90" step="1"
                        prop:value=move || deg.get().to_string()
                        attr:aria-valuetext=move || angle_label(deg.get())
                        on:input=move |ev| {
                            if let Ok(v) = event_target_value(&ev).parse::<f64>() {
                                set_deg.set(v);
                            }
                        }
                    />
                </label>
                <label class="slider">
                    <span class="row">
                        <span>"縦回転 ψ"</span>
                        <output>{move || pitch_label(pitch.get())}</output>
                    </span>
                    <input
                        type="range" min=(-PITCH_MAX).to_string() max=PITCH_MAX.to_string() step="1"
                        prop:value=move || pitch.get().to_string()
                        attr:aria-valuetext=move || pitch_label(pitch.get())
                        on:input=move |ev| {
                            if let Ok(v) = event_target_value(&ev).parse::<f64>() {
                                set_pitch.set(v);
                            }
                        }
                    />
                </label>
                <label class="slider">
                    <span class="row">
                        <span>"目の間隔"</span>
                        <output>{eye_x_label}</output>
                    </span>
                    <input
                        type="range" min=EYE_CENTER_X_MIN.to_string()
                        max=EYE_CENTER_X_MAX.to_string() step="0.5"
                        prop:value=move || eye_x.get().to_string()
                        attr:aria-valuetext=eye_x_label
                        on:input=move |ev| {
                            if let Ok(v) = event_target_value(&ev).parse::<f64>() {
                                set_eye_x.set(v);
                            }
                        }
                    />
                    <span class="hint">
                        {move || {
                            (use_image.get() && eye_x.get() > EYE_IMAGE_FIT_MAX)
                                .then_some("画像の目は、まつ毛の先が輪郭で欠け始めます")
                        }}
                    </span>
                </label>
                <button
                    type="button"
                    class="reset"
                    prop:disabled=move || eye_x.get() == EYE_CENTER_X
                    on:click=move |_| set_eye_x.set(EYE_CENTER_X)
                >
                    {format!("目の間隔を初期値（±{EYE_CENTER_X}）に戻す")}
                </button>
                <fieldset class="shape">
                    <legend>"断面の形（上から見た顔の前面）"</legend>
                    <label class="radio">
                        <input
                            type="radio" name="front"
                            prop:checked=move || wedge.get()
                            on:change=move |_| set_wedge.set(true)
                        />
                        "ゆるやかなV字"
                    </label>
                    <label class="radio">
                        <input
                            type="radio" name="front"
                            prop:checked=move || !wedge.get()
                            on:change=move |_| set_wedge.set(false)
                        />
                        "楕円（仕様）"
                    </label>
                </fieldset>
                <label class="slider" class:disabled=move || !wedge.get()>
                    <span class="row">
                        <span>"V字の傾き φ"</span>
                        <output>{move || format!("{:.0}°", phi.get())}</output>
                    </span>
                    <input
                        type="range" min=WEDGE_PHI_MIN.to_string() max=WEDGE_PHI_MAX.to_string()
                        step="1"
                        prop:value=move || phi.get().to_string()
                        prop:disabled=move || !wedge.get()
                        attr:aria-valuetext=move || format!("{:.0}°", phi.get())
                        on:input=move |ev| {
                            if let Ok(v) = event_target_value(&ev).parse::<f64>() {
                                set_phi.set(v);
                            }
                        }
                    />
                    <span class="hint muted">"0° で平らな顔、大きいほど鋭いV字"</span>
                </label>
                <label class="slider">
                    <span class="row">
                        <span>"奥行き倍率 q"</span>
                        <output>{move || format!("{:.2}倍", q.get())}</output>
                    </span>
                    <input
                        type="range" min="0.55" max="1.35" step="0.01"
                        prop:value=move || q.get().to_string()
                        attr:aria-valuetext=move || format!("{:.2}倍", q.get())
                        on:input=move |ev| {
                            if let Ok(v) = event_target_value(&ev).parse::<f64>() {
                                set_q.set(v);
                            }
                        }
                    />
                </label>
                <label class="check">
                    <input
                        type="checkbox"
                        prop:checked=move || show_ref.get()
                        on:change=move |ev| set_show_ref.set(event_target_checked(&ev))
                    />
                    "正面の輪郭を重ねる"
                </label>
                <label class="check">
                    <input
                        type="checkbox"
                        prop:checked=move || use_image.get()
                        on:change=move |ev| set_use_image.set(event_target_checked(&ev))
                    />
                    "目を画像素材で表示（オフで線画）"
                </label>
            </section>

            {move || match scene.get() {
                Err(e) => view! { <p class="error" role="alert">{e.to_string()}</p> }.into_any(),
                Ok(s) => {
                    view! { <Figures scene=s show_ref=show_ref.get() use_image=use_image.get() /> }
                        .into_any()
                }
            }}

            <section class="notes" aria-label="注記">
                <h2>"このモデルについて"</h2>
                <ul>
                    <li>"正面輪郭の左右幅は指定の輪郭画像から測った値です。前後の厚みは正面からは決まらないため設計値のままで、3D形状を推定したものではありません。"</li>
                    <li>"目の位置・前後の厚み・回転軸・顎の奥行きは説明用の設計値（仮定）です。"</li>
                    <li>"45°を超えると、輪郭は断面の計算結果から真横の手描きの横顔（唇・首の開口）へなめらかに変わります。これは見た目の指定に合わせる補正で、断面図と数値は断面モデルのままです。"</li>
                    <li>"頭頂の高さは、正面を含むどの角度でも真横の指定と同じです。正面の頭の上部は手描きの形で、45°にかけて計算上の形へ変わります。"</li>
                    <li>"縦回転 ψ は首の付け根を軸に、横回転より先に頭へ掛かります。真横では横顔がそのまま傾き、正面では輪郭が頭の外形に沿って変わります。断面図と数値は横回転だけの値です。"</li>
                    <li>"鼻は顔の中心の面にある線で、顔の上のレイヤーに描きます。正面では縦の直線で、回すと横に張り出し、真横で指定の形になります。"</li>
                    <li>"断面の前面は「ゆるやかなV字」（左右の角から中心線へ向かう2枚の平面）と「楕円（仕様）」を切り替えられます。V字でも中心線と後頭部の位置は楕円と同じで、左右の角が前に出るぶん、回したときに球のような丸い動きになりにくくなります。"</li>
                    <li>"鼻・耳・髪・口・首、眼球の立体、上下の回転、遠近法は省略しています（正射影）。"</li>
                    <li>{format!("目幅は y={EYE_Y} の目の横線のうち、見えている部分の投影幅を正面（{EYE_WIDTH:.1}）に対する比で示しています。")}</li>
                </ul>
            </section>
        </main>
    }
}

#[component]
fn Figures(scene: Scene, show_ref: bool, use_image: bool) -> impl IntoView {
    let s = scene;
    let [first, second] = ordered_eyes(&s);
    let status = format!(
        "見える目幅（正面比）：{} {}% ／ {} {}%",
        first.role.label(),
        first.percent.round(),
        second.role.label(),
        second.percent.round()
    );
    let b = s.section.bounds;

    view! {
        <div class="figures">
            <figure>
                <FaceView scene=s.clone() show_ref=show_ref use_image=use_image />
                <figcaption>"正面から見た顔（回転後）"</figcaption>
            </figure>
            <figure>
                <SectionView scene=s.clone() show_ref=show_ref />
                <figcaption>{format!("y={EYE_Y} の断面を上から見た図")}</figcaption>
            </figure>
        </div>

        <section class="stats" aria-label="数値">
            <p class="status" aria-live="polite">{status}</p>
            <dl>
                {[first, second]
                    .into_iter()
                    .map(|e| {
                        let cls = format!("stat {}", eye_class(e));
                        view! {
                            <div class=cls>
                                <dt>{format!("{}の目", e.role.label())}</dt>
                                <dd>
                                    <span class="big">{format!("{}%", e.percent.round())}</span>
                                    <span class="sub">
                                        {format!("幅 {:.1} / {:.1}", e.width, EYE_WIDTH)}
                                    </span>
                                </dd>
                            </div>
                        }
                    })
                    .collect_view()}
                <div class="stat">
                    <dt>"顎先の横ずれ"</dt>
                    <dd>
                        <span class="big">{format!("{:.1}", 60.0 * s.q * s.theta.sin())}</span>
                        <span class="sub">"60q·sinθ"</span>
                    </dd>
                </div>
                <div class="stat">
                    <dt>{format!("中心線→輪郭（y={EYE_Y}）")}</dt>
                    <dd>
                        <span class="big">
                            {format!("{:.1} | {:.1}", b.margin_left, b.margin_right)}
                        </span>
                        <span class="sub">"左 M_L ／ 右 M_R"</span>
                    </dd>
                </div>
            </dl>
        </section>
    }
}

#[component]
fn FaceView(scene: Scene, show_ref: bool, use_image: bool) -> impl IntoView {
    let s = scene;
    let eyes = if use_image {
        image_eyes(&s).into_any()
    } else {
        s.eyes
            .iter()
            .flat_map(|e| {
                let cls = eye_class(e);
                e.runs
                    .iter()
                    .map(move |run| view! { <path class=cls d=path_d(run, false) /> })
            })
            .collect_view()
            .into_any()
    };

    view! {
        <svg
            class="face"
            viewBox="-125 -30 250 245"
            role="img"
            aria-label=format!(
                "{}・{}に回転した顔",
                angle_label(s.degrees),
                pitch_label(s.head.pitch.to_degrees().round()),
            )
        >
            {show_ref.then(|| view! { <path class="reference" d=path_d(&s.reference, true) /> })}
            <path class="guide" d=path_d(&s.eye_level, false) />
            <path class="guide center" d=path_d(&s.centerline, false) />
            <path class="outline" d=path_d(&s.outline, false) />
            {eyes}
            <path class="nose-mask" d=path_d(&s.nose_mask, true) />
            <path class="nose" d=path_d(&s.nose, false) />
            <circle class="dot" cx=fmt(s.eye_level[0].0) cy=fmt(s.eye_level[0].1) r="1.5" />
            <circle class="dot" cx=fmt(s.eye_level[1].0) cy=fmt(s.eye_level[1].1) r="1.5" />
            <circle class="dot" cx=fmt(s.chin.0) cy=fmt(s.chin.1) r="1.7" />
        </svg>
    }
}

/// 目の画像素材を、表面に貼ったセルごとのアフィン変換で描く。
/// 各セルは内側の `<svg>` の viewport で画像を切り出す（画像 px 座標のまま）。
fn image_eyes(s: &Scene) -> impl IntoView {
    let cells = texture::cells(&EYES, s.theta, s.head, s.eye_x).unwrap_or_default();
    let (w, h) = (fmt(EYES.width), fmt(EYES.height));
    view! {
        <g class="eye-image">
            {cells
                .into_iter()
                .map(|c| {
                    let [a, b, cc, d, e, f] = c.matrix;
                    let [u, v, sw, sh] = c.src;
                    let transform = format!(
                        "matrix({:.5} {:.5} {:.5} {:.5} {:.4} {:.4})",
                        a, b, cc, d, e, f
                    );
                    view! {
                        <g transform=transform>
                            <svg
                                x=fmt(u) y=fmt(v) width=fmt(sw) height=fmt(sh)
                                viewBox=format!("{} {} {} {}", fmt(u), fmt(v), fmt(sw), fmt(sh))
                            >
                                <image href=EYES.href width=w.clone() height=h.clone() />
                            </svg>
                        </g>
                    }
                })
                .collect_view()}
        </g>
    }
}

#[component]
fn SectionView(scene: Scene, show_ref: bool) -> impl IntoView {
    let s = scene;
    let cs = &s.section;
    let b = cs.bounds;

    // 目の横線：見える部分は実線、裏側は破線で表示する（説明図なので裏側も描く）。
    let eye_lines = s
        .eyes
        .iter()
        .flat_map(|e| {
            let cls = eye_class(e);
            let proj = |p: &crate::model::SurfacePoint| (p.rotated.x, p.rotated.z);
            let vis: Vec<Polyline> = runs_where(&e.line, true, proj);
            let hid: Vec<Polyline> = runs_where(&e.line, false, proj);
            vis.into_iter()
                .map(move |r| {
                    view! { <path class=format!("eyeline {cls}") d=path_d(&r, false) /> }
                })
                .chain(hid.into_iter().map(move |r| {
                    view! { <path class=format!("eyeline hidden {cls}") d=path_d(&r, false) /> }
                }))
                .collect::<Vec<_>>()
        })
        .collect_view();

    // 投影幅：観察者側の基準線に、輪郭幅と各目の見える幅を落とす。
    let spans = s
        .eyes
        .iter()
        .enumerate()
        .filter_map(|(i, e)| {
            let (lo, hi) = e.span?;
            let z = PROJ_Z + 5.0 + 4.0 * i as f64;
            let cls = eye_class(e);
            Some(view! {
                <path
                    class=format!("span {cls}")
                    d=path_d(&[(lo, z), (hi, z)], false)
                />
            })
        })
        .collect_view();

    let projectors = cs
        .contour_now
        .iter()
        .map(|&(x, z)| {
            view! { <path class="projector" d=path_d(&[(x, z), (x, PROJ_Z)], false) /> }
        })
        .collect_view();

    let dots = cs
        .contour_now
        .iter()
        .map(|&(x, z)| view! { <circle class="dot" cx=fmt(x) cy=fmt(z) r="2.4" /> })
        .collect_view();
    let squares = cs
        .contour_front
        .iter()
        .map(|&(x, z)| {
            view! {
                <rect
                    class="square"
                    x=fmt(x - 2.4) y=fmt(z - 2.4) width="4.8" height="4.8"
                />
            }
        })
        .collect_view();

    view! {
        <svg
            class="section"
            viewBox="-115 -108 230 250"
            role="img"
            aria-label="目の高さの断面を上から見た図"
        >
            {show_ref.then(|| view! { <path class="reference" d=path_d(&cs.reference, true) /> })}
            <path class="guide" d="M-6,0L6,0M0,-6L0,6" />
            {projectors}
            <path class="outline" d=path_d(&cs.outline, true) />
            {eye_lines}
            <circle class="center-mark" cx=fmt(cs.center.0) cy=fmt(cs.center.1) r="1.6" />
            {squares}
            {dots}
            <path class="baseline" d=path_d(&[(b.left, PROJ_Z), (b.right, PROJ_Z)], false) />
            {spans}
            <text x="0" y="132" text-anchor="middle">"見る方向 ↑"</text>
        </svg>
        <ul class="legend">
            <li><span class="key dot-key"></span>"今の輪郭の点"</li>
            <li><span class="key square-key"></span>"正面で輪郭だった点"</li>
            <li><span class="key dash-key"></span>"裏側（見えない）の目"</li>
            <li><span class="key bar-key"></span>"投影幅"</li>
        </ul>
    }
}
