VERSION=0.2.0

all: mcp-${VERSION}.pkg

files:
	mkdir -p files/usr/local/bin
	cp ../../target/release/mcp files/usr/local/bin

resources:
	mkdir resources
	sed "s/\$$VERSION/${VERSION}/g" welcome.html.template > resources/welcome.html
	cp license.html resources/

mcp.pkg: files
	pkgbuild --root files --identifier org.github.apoloval.mcp --version ${VERSION} mcp.pkg

mcp-${VERSION}.pkg: mcp.pkg distribution.xml resources
	productbuild --distribution distribution.xml --resources resources mcp-${VERSION}.pkg

.PHONY clean:
	rm -rf files resources *.pkg
