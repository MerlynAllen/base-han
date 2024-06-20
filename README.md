# Base-CJK

Use CJK characters to encode to binary data to text.

**CJK chracter ranges:**
<table>
  <tr>
    <th style="text-align: left">CJK Unified Ideographs</th>
    <td>4E00-9FFF</td>
    <td>Common</td>
  </tr>
  <tr>
    <th style="text-align: left">CJK Unified Ideographs Extension A</th>
    <td>3400-4DBF</td>
    <td>Rare</td>
  </tr>
</table>
This utility converts every 13 bits to a Unicode code point 
which lies in the range of `[4E00, 6E00)`.
In addition, 8E00 is also used as a functional character to 
show whether the ending byte is split to 2 code points or not.
In v1, in order to support streaming mode, we make `[6E00, 7E00)`
in use, which has 2^12 code points, to indicate the end of file
without introducing control characters with no information, 
which requires the decoder to peek 1 character forward while 
somehow impossible in streaming.