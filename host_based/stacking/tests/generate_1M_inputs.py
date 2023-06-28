import random


def generate_1M_inputs():
    """Generate 1M inputs for the test."""
    with open('1M_inputs.sh', 'w') as f:
        for _ in range(1000000):
            ch = "".join([f" {random.randint(0, 255):02x}" for _ in range(16)])
            f.write(f"echo \"{ch[1:]}\"\n")

if __name__ == '__main__':
    generate_1M_inputs()