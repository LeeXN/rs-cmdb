use yew::prelude::*;
use web_sys::{CanvasRenderingContext2d, HtmlCanvasElement};
use wasm_bindgen::JsCast;
use std::f64::consts::PI;

#[derive(Debug, Clone, PartialEq)]
pub struct ChartData {
    pub label: String,
    pub value: f64,
    pub color: String,
}

#[derive(Debug, Clone, PartialEq, Properties)]
pub struct PieChartProps {
    pub data: Vec<ChartData>,
    #[prop_or(300)]
    pub width: u32,
    #[prop_or(300)]
    pub height: u32,
    #[prop_or_default]
    pub title: Option<String>,
}

#[function_component(PieChart)]
pub fn pie_chart(props: &PieChartProps) -> Html {
    let canvas_ref = use_node_ref();

    {
        let canvas_ref = canvas_ref.clone();
        let data = props.data.clone();
        let width = props.width;
        let height = props.height;
        use_effect_with((data.clone(),), move |_| {
            if let Some(canvas) = canvas_ref.cast::<HtmlCanvasElement>() {
                if let Ok(context) = canvas
                    .get_context("2d")
                    .unwrap()
                    .unwrap()
                    .dyn_into::<CanvasRenderingContext2d>()
                {
                    draw_pie_chart(&context, &data, width as f64, height as f64);
                }
            }
            || ()
        });
    }

    html! {
        <div class="chart-container">
            {
                if let Some(title) = &props.title {
                    html! { <h6 class="text-center mb-3 font-bold text-lg">{title}</h6> }
                } else {
                    html! {}
                }
            }
            <canvas 
                ref={canvas_ref}
                width={props.width.to_string()}
                height={props.height.to_string()}
                style="max-width: 100%; height: auto;"
            />
        </div>
    }
}

#[derive(Debug, Clone, PartialEq, Properties)]
pub struct BarChartProps {
    pub data: Vec<ChartData>,
    #[prop_or(400)]
    pub width: u32,
    #[prop_or(300)]
    pub height: u32,
    #[prop_or_default]
    pub title: Option<String>,
}

#[function_component(BarChart)]
pub fn bar_chart(props: &BarChartProps) -> Html {
    let canvas_ref = use_node_ref();

    {
        let canvas_ref = canvas_ref.clone();
        let data = props.data.clone();
        let width = props.width;
        let height = props.height;
        use_effect_with((data.clone(),), move |_| {
            if let Some(canvas) = canvas_ref.cast::<HtmlCanvasElement>() {
                if let Ok(context) = canvas
                    .get_context("2d")
                    .unwrap()
                    .unwrap()
                    .dyn_into::<CanvasRenderingContext2d>()
                {
                    draw_bar_chart(&context, &data, width as f64, height as f64);
                }
            }
            || ()
        });
    }

    html! {
        <div class="chart-container">
            {
                if let Some(title) = &props.title {
                    html! { <h6 class="text-center mb-3 font-bold text-lg">{title}</h6> }
                } else {
                    html! {}
                }
            }
            <canvas 
                ref={canvas_ref}
                width={props.width.to_string()}
                height={props.height.to_string()}
                style="max-width: 100%; height: auto;"
            />
        </div>
    }
}

fn draw_pie_chart(context: &CanvasRenderingContext2d, data: &[ChartData], width: f64, height: f64) {
    // 清除画布
    context.clear_rect(0.0, 0.0, width, height);
    
    if data.is_empty() {
        return;
    }

    let center_x = width / 2.0;
    let center_y = height / 2.0;
    let radius = (width.min(height) / 2.0) * 0.8;

    let total: f64 = data.iter().map(|d| d.value).sum();
    if total == 0.0 {
        return;
    }

    let mut current_angle = -PI / 2.0; // 从顶部开始

    for item in data {
        let slice_angle = (item.value / total) * 2.0 * PI;
        
        // 绘制扇形
        context.begin_path();
        context.move_to(center_x, center_y);
        context.arc(center_x, center_y, radius, current_angle, current_angle + slice_angle).unwrap();
        context.close_path();
        
        // 设置填充样式
        let _ = context.set_fill_style_str(&item.color);
        context.fill();
        
        // 绘制边框
        let _ = context.set_stroke_style_str("#ffffff");
        context.set_line_width(2.0);
        context.stroke();

        current_angle += slice_angle;
    }
}

fn draw_bar_chart(context: &CanvasRenderingContext2d, data: &[ChartData], width: f64, height: f64) {
    // 清除画布
    context.clear_rect(0.0, 0.0, width, height);
    
    if data.is_empty() {
        return;
    }

    let margin = 40.0;
    let chart_width = width - 2.0 * margin;
    let chart_height = height - 2.0 * margin;
    
    let max_value = data.iter().map(|d| d.value).fold(0.0, f64::max);
    if max_value == 0.0 {
        return;
    }

    let bar_width = chart_width / data.len() as f64 * 0.8;
    let bar_spacing = chart_width / data.len() as f64 * 0.2;

    // 绘制坐标轴
    let _ = context.set_stroke_style_str("#666666");
    context.set_line_width(1.0);
    
    // Y轴
    context.begin_path();
    context.move_to(margin, margin);
    context.line_to(margin, height - margin);
    context.stroke();
    
    // X轴
    context.begin_path();
    context.move_to(margin, height - margin);
    context.line_to(width - margin, height - margin);
    context.stroke();

    // 绘制柱状图
    for (i, item) in data.iter().enumerate() {
        let x = margin + i as f64 * (bar_width + bar_spacing) + bar_spacing / 2.0;
        let bar_height = (item.value / max_value) * chart_height;
        let y = height - margin - bar_height;

        let _ = context.set_fill_style_str(&item.color);
        context.fill_rect(x, y, bar_width, bar_height);
        
        // 绘制边框
        let _ = context.set_stroke_style_str("#ffffff");
        context.set_line_width(1.0);
        context.stroke_rect(x, y, bar_width, bar_height);
    }
}

// 预定义的颜色方案
// pub fn get_chart_colors() -> Vec<String> {
//     vec![
//         "#FF6384".to_string(),
//         "#36A2EB".to_string(),
//         "#FFCE56".to_string(),
//         "#4BC0C0".to_string(),
//         "#9966FF".to_string(),
//         "#FF9F40".to_string(),
//         "#FF6384".to_string(),
//         "#C9CBCF".to_string(),
//         "#4BC0C0".to_string(),
//         "#FF6384".to_string(),
//     ]
// } 