
    // Use gettimeofday to try to find the cache line size by sampling.
    // Idea by Rudi Cilibrasi.
    // Compile with gcc testcache.c -o testcache -lm
    #include <stdio.h>
    #include <math.h>
    #include <stdlib.h>
    #include <string.h>
    #include <sys/time.h>
    // We aim to detect cache lines in the 8-4096 size range.
    #define MAXPOWTWO 15
    #define MAXLINESIZE (1 << MAXPOWTWO)
    // Increase these numbers to slow down the program and increase reliability.
    #define STEPCOUNT 500
    #define BIGLOOPCOUNT 500
    double gettime(void) {
      struct timeval tv;
      gettimeofday(&tv, NULL);
      return (double) (tv.tv_sec) + (double) (tv.tv_usec * 1e-6);
    }
    char bytes[(STEPCOUNT+1)*MAXLINESIZE];
    // Try to find the time taken to do STEPCOUNT skips forward:
    // Each step move forward skip_length bytes.
    double runTrial(int skip_length) {
      memset(bytes, 1, (STEPCOUNT+1)*MAXLINESIZE);
      int i, k = 0;
      double t1 = gettime();
      for (i = 0; i < STEPCOUNT; ++i) {
        bytes[k] = 3;
        k += skip_length;
      }
      double t2 = gettime();
      return t2 - t1;
    }
    int main(int argc, char **argv)
    {
      int i, j;
      int probes_size[MAXPOWTWO];
      double timings[MAXPOWTWO], norm[MAXPOWTWO];
      memset(timings, 0, sizeof(timings));
      for (j = 0; j < BIGLOOPCOUNT; ++j) {
        for (i = 0; i < MAXPOWTWO; ++i) {
          int psize = 1 << i;
          probes_size[i] = psize;
          timings[i] += runTrial(psize);
        }
      }
      for (i = 0; i < MAXPOWTWO; ++i) {
        norm[i] = 1000*timings[i] / pow(probes_size[i], 2.0/3.0);
      }
      // Find first local minimum
      double minval = 1e6;
      int bestind = 0;
      for (i = 1; i < MAXPOWTWO; ++i) {
        if (norm[i] > norm[i-1]) {
          bestind = i;
          break;
        }
      }
      // Find local maximum after the local minimum
      double maxval = 0;
      for (i = bestind; i < MAXPOWTWO-1; ++i) {
        if (norm[i] > norm[i+1]) {
          bestind = i;
          break;
        }
      }
      printf("%d\n", probes_size[bestind]);
      exit(0);
    }
