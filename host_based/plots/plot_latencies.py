import matplotlib.pyplot as plt
import sys
import samples

if __name__ == "__main__":


    # import module in runtime from sys.argv


    for l in samples.latencies:
        plt.plot(
                list(range(len(l))), l, '.', color='red', alpha=0.2
            )
    # plt.hist(samples.latencies)
    plt.show()
