# What's this?

`freak` is a cli frequency analysis tool. You can use it to check the frequency distribution of f.ex. a text (which would show you how many times each letter/byte occurs.) This useful if you have some data which you have no clue what is. Some types of encoding preserve the frequency distribution or transform it in predictable ways. Compression and encryption makes distributions uniform, base64 encoding limits the distribution to 64 different bytes (instead of 256), ect.

![freak being run on a poem called The Conscience of a Hacker. It shows a bargraph with the most frequent byte being space or hex 20.](/images/example.png)

# TODO
 - [x] Sort by frequency (i.e. If there are more 'e's those come before 'c')
 - [ ] Option to sort by hex value (i.e. 0x20 comes before 0x21)
 - [x] Bar graph
 - [x] Print hex and ASCII
 - [ ] Enable/disable ASCII print
 - [ ] Scriptable output ?? need to workshop this
 - [ ] Classify data by frequency analysis
 - [ ] Add metrics like index of coincidence + others
 - [ ] Option to show percentage or total count
 - [ ] Option to enable/disable UTF-8 / ASCII only graphics
 - [ ] Figure out some way to put color into this bad boy
 - [ ] Analyze unicode code points / graphemes instead of bytes?
 - [ ] Add vertical bargraph (where the bars are horizontal, and the next bar is vertically down)

# Issues
 - [ ] Refractor printing into a seperate method
 - [ ] ASCII printing is printing non-ascii characters like `ð`
 - [ ] Make the bargraph responsive to your screen in a bunch of ways
 - [ ] Frequency analysis seems slow, 5.2MB takes 2s