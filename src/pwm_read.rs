use stm32f4xx_hal::{
    stm32::{RCC, TIM8},
    timer::{PinC1, PinC2},
};

pub struct PwmInput<TIM, PINS> {
    tim: TIM,
    pins: PINS,
}

pub trait Pins<TIM> {}


// implement the `Pins` trait wherever PC1 implements PinC1 and PC2 implements PinC2 for the given TIMer
impl<TIM, PC1> Pins<TIM> for PC1
where
    PC1: PinC1<TIM>,
{
}

impl<PINS> PwmInput<TIM8, PINS> {
    /// Configures a TIM peripheral as a PWM input
    pub fn tim8(tim: TIM8, pins: PINS) -> Self
    where
        PINS: Pins<TIM8>,
    {
        // NOTE(unsafe) this reference will only be used for atomic writes with no side effects.
        let rcc = unsafe { &(*RCC::ptr()) };
        // enable and reset clock.
        rcc.apb2enr.modify(|_, w| w.tim8en().set_bit());
        rcc.apb2rstr.modify(|_, w| w.tim8rst().set_bit());
        rcc.apb2rstr.modify(|_, w| w.tim8rst().clear_bit());

        // Select the active input for TIMx_CCR1: write the CC1S bits to 01 in the TIMx_CCMR1
        // register (TI1 selected).

        tim.ccmr1_input().modify(|_,w| {
            w.cc1s().ti1()
        });

        // Select the active polarity for TI1FP1 (used both for capture in TIMx_CCR1 and counter
        // clear): write the CC1P and CC1NP bits to ‘0’ (active on rising edge).

        tim.ccer.modify(|_, w| {
            w.cc1p().clear_bit().cc2p().clear_bit()
        } );

        // Select the active input for TIMx_CCR2: write the CC2S bits to 10 in the TIMx_CCMR1
        // register (TI1 selected)
        tim.ccmr1_input().modify(|_, w| {
            w.cc2s().ti1()
        });

        // Select the active polarity for TI1FP2 (used for capture in TIMx_CCR2): write the CC2P
        // and CC2NP bits to ‘1’ (active on falling edge).
        tim.ccer.modify(|_, w| {
            w.cc2p().set_bit().cc2np().set_bit()
        });

        // Select the valid trigger input: write the TS bits to 101 in the TIMx_SMCR register
        // (TI1FP1 selected).
        tim.smcr.modify(|_, w| {
            w.ts().ti1fp1()
        });

        // Configure the slave mode controller in reset mode: write the SMS bits to 100 in the
        // TIMx_SMCR register.
        tim.smcr.modify(|_, w| {
            w.sms().reset_mode()
        }
        );

        // Enable the captures: write the CC1E and CC2E bits to ‘1’ in the TIMx_CCER register.
        tim.ccer.modify(|_, w| {
            w.cc1e().set_bit().cc2e().set_bit()
        });

        // enable interrupts.
        tim.dier.modify(|_, w| {
            w.cc2ie().set_bit()
        });
        PwmInput { tim, pins }
    }

    /// Releases the TIM peripheral and QEI pins
    pub fn release(self) -> (TIM8, PINS) {
        (self.tim, self.pins)
    }

    pub fn get_period(&self) -> u16 {
        self.tim.ccr1.read().ccr().bits()

    }
    pub fn get_duty_cycle(&self) -> u16 {
        self.tim.ccr2.read().ccr().bits()
    }
}
