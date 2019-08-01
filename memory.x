/* Linker script for building examples for the STM32F429ZI */
MEMORY
{
  FLASH : ORIGIN = 0x08000000, LENGTH = 2048K
  RAM : ORIGIN = 0x20000000, LENGTH = 192K
}
