echo "                             "
echo "++++++++ BUILDING +++++++++++"
echo "                             "
cargo build --release
echo "                             "
echo "++++++++ DELETING OLD BINARY +++++++++++"
echo "                             "
rm -rf ~/.local/bin/transent
echo "                             "
echo "++++++++ MOVING THE NEWLY BUILD BINARY TO PATH +++++++++++"
echo "                             "
cp -r ./target/release/transent ~/.local/bin/
