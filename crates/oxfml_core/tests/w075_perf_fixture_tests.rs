use std::collections::BTreeMap;
use std::time::{Duration, Instant};

mod common;

use oxfml_core::eval::{EvaluationContext, evaluate_formula};
use oxfunc_core::value::EvalValue;

const MANDELBROT_100_60_30: &str = r#"=LET(
  rows, 100,
  cols, 60,
  maxIter, 30,
  cx, -0.5,
  cy, 0,
  zoom, 1.2,
  width, 3 / zoom,
  height, 2.4 / zoom,
  palette, " .:-=+*#%@",
  rowSeq, SEQUENCE(rows, 1, 0, 1),
  colSeq, SEQUENCE(1, cols, 0, 1),
  x0, cx - width/2 + (colSeq / (cols - 1)) * width,
  y0, cy - height/2 + (rowSeq / (rows - 1)) * height,
  mandel, LAMBDA(a,b,
    REDUCE(
      HSTACK(0, 0, 0),
      SEQUENCE(maxIter),
      LAMBDA(state,k,
        LET(
          x, INDEX(state, 1, 1),
          y, INDEX(state, 1, 2),
          n, INDEX(state, 1, 3),
          escaped, (x*x + y*y) > 4,
          IF(escaped,
             state,
             HSTACK(x*x - y*y + a, 2*x*y + b, n + 1)
          )
        )
      )
    )
  ),
  iters, MAKEARRAY(rows, cols, LAMBDA(r,c,
    INDEX(mandel(INDEX(x0, 1, c), INDEX(y0, r, 1)), 1, 3)
  )),
  charIdx, IF(iters = maxIter, 1, 1 + INT(iters / maxIter * (LEN(palette) - 1))),
  MID(palette, charIdx, 1)
)"#;

#[test]
#[ignore = "manual W075 release-mode timing fixture; run with --ignored --nocapture"]
fn w075_manual_hot_loop_perf_fixture() {
    let cases = [
        ("makearray_constant", "=MAKEARRAY(100,60,LAMBDA(r,c,1))", 3),
        (
            "reduce_scalar",
            "=REDUCE(0,SEQUENCE(6000),LAMBDA(a,b,a+b))",
            3,
        ),
        (
            "reduce_state_index_hstack",
            "=REDUCE(HSTACK(0,0,0),SEQUENCE(6000),LAMBDA(state,k,HSTACK(INDEX(state,1,1),INDEX(state,1,2),INDEX(state,1,3))))",
            3,
        ),
        ("mandelbrot_100_60_30", MANDELBROT_100_60_30, 3),
    ];

    for (case_id, formula, measured_runs) in cases {
        let compiled = common::compile_formula(
            &format!("w075-{case_id}"),
            formula,
            BTreeMap::new(),
            "w075-struct-v1",
            "oxfunc:w075",
        );

        let warmup = evaluate_once(&compiled).expect("warmup evaluation should succeed");
        let mut timings = Vec::with_capacity(measured_runs);
        let mut result_summary = summarize_eval_value(&warmup.oxfunc_value);
        for _ in 0..measured_runs {
            let started = Instant::now();
            let output = evaluate_once(&compiled).expect("measured evaluation should succeed");
            timings.push(started.elapsed());
            result_summary = summarize_eval_value(&output.oxfunc_value);
        }

        let min = timings.iter().min().copied().unwrap_or_default();
        let max = timings.iter().max().copied().unwrap_or_default();
        let avg = average_duration(&timings);
        eprintln!(
            "W075_PERF case={case_id} runs={measured_runs} min_ms={:.3} avg_ms={:.3} max_ms={:.3} result={result_summary}",
            duration_ms(min),
            duration_ms(avg),
            duration_ms(max),
        );
    }
}

fn evaluate_once(
    compiled: &common::CompiledFormulaArtifacts,
) -> Result<oxfml_core::EvaluationOutput, oxfml_core::EvaluationError> {
    let context = EvaluationContext::new(&compiled.bound_formula, &compiled.semantic_plan);
    evaluate_formula(context)
}

fn average_duration(durations: &[Duration]) -> Duration {
    if durations.is_empty() {
        return Duration::default();
    }
    let total_nanos: u128 = durations.iter().map(Duration::as_nanos).sum();
    Duration::from_nanos((total_nanos / durations.len() as u128) as u64)
}

fn duration_ms(duration: Duration) -> f64 {
    duration.as_secs_f64() * 1000.0
}

fn summarize_eval_value(value: &EvalValue) -> String {
    match value {
        EvalValue::Array(array) => {
            let shape = array.shape();
            format!("Array({}x{})", shape.rows, shape.cols)
        }
        EvalValue::Number(number) => format!("Number({number})"),
        EvalValue::Text(text) => format!("Text(len={})", text.to_string_lossy().chars().count()),
        EvalValue::Logical(value) => format!("Logical({value})"),
        EvalValue::Error(code) => format!("Error({code:?})"),
        EvalValue::Reference(reference) => format!("Reference({})", reference.target),
        other => format!("Unsupported({other:?})"),
    }
}
