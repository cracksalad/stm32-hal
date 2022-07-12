//! This module includes an overview of timer
//! For project structure and debugging boilerplate, see the `synax_overview` example.

#![no_main]
#![no_std]

use cortex_m::{
    interrupt::{free, Mutex},
    peripheral::NVIC,
};
use cortex_m_rt::entry;

use stm32_hal2::{
    clocks::Clocks,
    gpio::{Edge, Pin, PinMode, Port},
    low_power, pac,
    timer::{
        Alignment, BasicTimer, CaptureCompare, CountDir, InputSlaveMode, InputTrigger,
        MasterModeSelection, OutputCompare, TimChannel, Timer, TimerConfig, TimerInterrupt,
    },
};

#[entry]
fn main() -> ! {
    // Set up microcontroller peripherals
    let mut dp = pac::Peripherals::take().unwrap();

    let clock_cfg = Clocks::default();

    clock_cfg.setup().unwrap();

    // Set up a PWM pin
    let _pwm_pin = Pin::new(Port::A, 0, PinMode::Alt(1));

    // Set up a PWM timer that will output to PA0, run at 2400Hz in edge-aligned mode,
    // count up, with a 50% duty cycle.
    let mut pwm_timer = Timer::new_tim2(
        dp.TIM2,
        2_400.,
        TimerConfig {
            auto_reload_preload: true,
            // Setting auto reload preload allow changing frequency (period) while the timer is running.
            ..Default::default()
        },
        &clock_cfg,
    );

    // Example syntax to set up PWM output on channel1, with a 50% duty cycle.
    pwm_timer.enable_pwm_output(TimChannel::C1, OutputCompare::Pwm1, 0.5);

    // Example syntax for enabling input capture. Eg, for PWM input
    pwm_timer.set_input_capture(
        TimChannel::C2,
        CaptureCompare::InputTi1,
        InputTrigger::Internal0,
        InputSlaveMode::Disabled,
        true,
        false,
    );

    pwm_timer.enable();

    // Change the duty cycle. The argument is the auto-reload value (ARR).
    pwm_timer.set_duty(TimChannel::C1, 100);

    // Exampler of more settings
    let timer_config = TimerConfig {
        one_pulse_mode: true,
        alignment: Alignment::Edge,
        direction: CountDir::Down,
        ..Default::default()
    };

    let period = 2.; // seconds.
    let mut countdown_timer = Timer::new_tim3(dp.TIM3, countdown_period, timer_config, &clock_cfg);
    countdown_timer.enable_interrupt(TimerInterrupt::Update); // Enable update event interrupts.
    countdown_timer.enable();

    // Change the frequency to 1Khz.
    pwm_timer.set_freq(1_000.);

    // Or set PSC and ARR manually, eg to set period (freq), without running the calculations
    // used in `set_freq`.
    pwm_timer.set_auto_reload(100);
    pwm_timer.set_prescaler(100);

    let seconds_elapsed = countdown_timer.read_count() / countdown_timer.get_max_duty() * period;
    println!("Time elapsed since timer start: {}", seconds_elapsed);

    // Set up a basic timer, eg for DAC triggering
    let mut dac_timer = BasicTimer::new(
        dp.TIM6,
        clock_cfg.sai1_speed() as f32 / (64. * 8.),
        &clock_cfg,
    );

    //  The update event is selected as a trigger output (TRGO). For instance a
    // master timer can then be used as a prescaler for a slave timer.
    dac_timer.set_mastermode(MasterModeSelection::Update);

    // todo: Burst DMA example, realistic examples of various uses of timers etc.

    // Unmask the interrupt line.
    unsafe {
        NVIC::unmask(pac::Interrupt::TIM3);
    }

    loop {
        low_power::sleep_now();
    }
}

#[interrupt]
/// Timer interrupt handler; runs when the countdown period expires.
fn TIM3() {
    free(|cs| {
        // Clear the interrupt flag. If you ommit this, it will fire repeatedly.
        unsafe { (*pac::TIM3::ptr()).sr.modify(|_, w| w.uif().clear_bit()) }
        // If you have access to the timer variable, eg through a Mutex, you can do this instead:
        // countdown_timer_clear_interrupt(TimerInterrupt::Update);

        defmt::println!("Countdown expired");
    });

    // Do something.
}

// same panicking *behavior* as `panic-probe` but doesn't print a panic message
// this prevents the panic message being printed *twice* when `defmt::panic` is invoked
#[defmt::panic_handler]
fn panic() -> ! {
    cortex_m::asm::udf()
}
