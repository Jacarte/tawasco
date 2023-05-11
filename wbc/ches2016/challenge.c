#include <stdio.h>
#include <stdlib.h>
#include "chow_aes3_encrypt_wb.h"

const int BLOCK_SIZE = 16;

int main(int argc, char **argv)
{
  unsigned char input[BLOCK_SIZE];
  unsigned char out[BLOCK_SIZE];
  int i;

  if (argc != 17)
  {
    printf("Usage:\n %s <16 bytes in hex, separated by spaces>\n", argv[0]);
    return 1;
  }

  // Read the input data.
  for (i = 0; i < BLOCK_SIZE; i++)
  {
    input[i] = strtoul(argv[i + 1], NULL, 16);
  }

  // Encryption
  chow_aes3_encrypt_wb(input, out);

  // Print input and output texts
  printf("INPUT:     ");
  for (i = 0; i < BLOCK_SIZE; i++)
  {
    printf("%02x ", input[i]);
  }
  printf("\nOUTPUT:    ");
  for (i = 0; i < BLOCK_SIZE; i++)
  {
    printf("%02x ", out[i]);
  }
  printf("\n");
  return 0;
}
