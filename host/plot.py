import samples
import matplotlib.pyplot as plt
import samples


if __name__ == "__main__":


    #for l in samples.latencies:
    l  = samples.latencies
    plt.scatter(list(range(len(l))), l, color='red', alpha=0.1)

    # plt.hist(l, bins=len(set(l)))
    # plot bar for heach of the scores values
    scores = samples.scores
    # plt.bar(list(range(len(scores))), scores, color='blue', alpha=0.1)
    plt.show()
