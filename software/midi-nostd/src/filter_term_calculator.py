import math

def init_lowpass_butterworth(cutoff_hz: float, sample_rate: float) -> tuple[float, float, float, float, float]:
    """
    Calculates the coefficients for a second-order (12 dB/octave) Butterworth
    low-pass filter using the bilinear transform method.

    Args:
        cutoff_hz: The desired cutoff frequency in Hertz.
        sample_rate: The audio sample rate in Hertz.

    Returns:
        A tuple containing the filter coefficients (b0, b1, b2, a1, a2).
    """
    # Bilinear Transform variable
    K = math.tan(math.pi * cutoff_hz / sample_rate)
    K_squared = K * K
    sqrt2 = math.sqrt(2.0)

    # Denominator for coefficient normalization
    a0_denom = 1.0 + sqrt2 * K + K_squared
    
    # Numerator coefficients
    b0 = K_squared / a0_denom
    b1 = 2.0 * K_squared / a0_denom
    b2 = K_squared / a0_denom
    
    # Denominator coefficients (for feedback)
    a1 = 2.0 * (K_squared - 1.0) / a0_denom
    a2 = (1.0 - sqrt2 * K + K_squared) / a0_denom
    
    return b0, b1, b2, a1, a2

# --- Example Usage ---
if __name__ == "__main__":
    sample_rate = 24000.0  # Common audio sample rate
    cutoff_frequency = 150.0 # 40 Hz

    print(f"--- 12 dB/octave Filter Coefficients ---")
    print(f"A single biquad section (Cutoff: {cutoff_frequency} Hz, Sample Rate: {sample_rate} Hz)")
    
    b0_12db, b1_12db, b2_12db, a1_12db, a2_12db = init_lowpass_butterworth(cutoff_frequency, sample_rate)

    print(f"b0: {b0_12db:.10f}")
    print(f"b1: {b1_12db:.10f}")
    print(f"b2: {b2_12db:.10f}")
    print(f"a1: {a1_12db:.10f}")
    print(f"a2: {a2_12db:.10f}")

    b0_composed = b0_12db;
    b1_composed = b1_12db - a1_12db - 1.0;
    b2_composed = b2_12db - a2_12db;

    print(f"b0: {b0_composed:.10f}")
    print(f"b1: {b1_composed:.10f}")
    print(f"b2: {b2_composed:.10f}")

    b0_int = int(b0_composed * float(1<<31))
    b1_int = int(b1_composed * float(1<<31))
    b2_int = int(b2_composed * float(1<<31))

    print(f"b0_int: {b0_int:.10f}")
    print(f"b1_int: {b1_int:.10f}")
    print(f"b2_int: {b2_int:.10f}")

    print(f"\n--- 24 dB/octave Filter Coefficients ---")
    print(f"Two cascaded biquad sections. The coefficients for each section are the same.")
    print(f"You would use the same set of coefficients for both stages.")
    
    # The coefficients for a 24dB/octave filter are just two sets of the 12dB coefficients
    b0_24db, b1_24db, b2_24db, a1_24db, a2_24db = init_lowpass_butterworth(cutoff_frequency, sample_rate)

    print(f"Set 1 (for first stage):")
    print(f"b0: {b0_24db:.10f}")
    print(f"b1: {b1_24db:.10f}")
    print(f"b2: {b2_24db:.10f}")
    print(f"a1: {a1_24db:.10f}")
    print(f"a2: {a2_24db:.10f}")

    print(f"\nSet 2 (for second stage):")
    print(f"b0: {b0_24db:.10f}")
    print(f"b1: {b1_24db:.10f}")
    print(f"b2: {b2_24db:.10f}")
    print(f"a1: {a1_24db:.10f}")
    print(f"a2: {a2_24db:.10f}")

    # You can also compute for a different cutoff frequency
    cutoff_frequency_high = 200.0
    b0_high, b1_high, b2_high, a1_high, a2_high = init_lowpass_butterworth(cutoff_frequency_high, sample_rate)

    print(f"\n--- Example with a 200 Hz cutoff ---")
    print(f"b0: {b0_high:.10f}")
    print(f"b1: {b1_high:.10f}")
    print(f"b2: {b2_high:.10f}")
    print(f"a1: {a1_high:.10f}")
    print(f"a2: {a2_high:.10f}")

