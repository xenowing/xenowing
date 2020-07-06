## Common parameters

All buses have the following parameters that apply to each relevant port on that bus:

| name | description |
| --- | --- |
| `data_bit_width` | The width of the port datapaths for this bus. Must be a multiple of 8 (byte-oriented). This can vary between buses, but all datapaths for a given port must have the same width. |
| `addr_bit_width` | The width of the address datapath for this bus. |

## Common signals

All ports have at least the following signals:

| name | bit width | direction (from primary) | description |
| --- | --- | --- | --- |
| `bus_ready` | 1 | in | replica is ready to accept a transaction |
| `bus_enable` | 1 | out | indicates a transaction |
| `bus_addr` | `addr_bit_width` | out | the address of a transaction, in units of `data_bit_width` bits |

## Read signals

Ports with a read channel add the following signals:

| name | bit width | direction (from primary) | description |
| --- | --- | --- | --- |
| `bus_read_data` | `data_bit_width` | in | data returned from a read transaction |
| `bus_read_data_valid` | 1 | in | signals that there's data returned from a read transaction present in `bus_read_data` |

## Write signals

Ports with a write channel add the following signals:

| name | bit width | direction (from primary) | description |
| --- | --- | --- | --- |
| `bus_write_data` | `data_bit_width` | out | data sent for a write transaction |
| `bus_write_byte_enable` | `data_bit_width / 8` | out | indicates which 8-bit chunks of `bus_write_data` should be updated in this write transaction (high = update, low = leave as-is) |

## Read/write signals

Ports with both a read and write channel add the following signals:

| name | bit width | direction (from primary) | description |
| --- | --- | --- | --- |
| `bus_write` | 1 | out | indicates whether a transaction is a write transaction (high) or a read transaction (low) |

