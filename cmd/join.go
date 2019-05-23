package cmd

import (
	"crypto/rand"
	"crypto/rsa"
	"crypto/x509"
	"encoding/pem"
	"fmt"
	"log"
	"math/big"
	"net/http"
	"os"
	"path/filepath"
	"time"

	"github.com/shibukawa/configdir"
	"github.com/spf13/cobra"
)

func init() {
	rootCmd.AddCommand(joinCmd)
}

var joinCmd = &cobra.Command{
	Use: "join",
	Run: func(cmd *cobra.Command, args []string) {
		join()
	},
}

func join() {
	// usr, err := user.Current()
	// usr.
	// os.UserCacheDir()
	fmt.Printf("Hello, world!\n")
	certFile, keyFile, err := makeCert()
	if err != nil {
		log.Fatal(err)
	}
	fmt.Printf("%s\n", certFile)
	http.HandleFunc("/", handler)
	// log.Fatal(http.ListenAndServe(":8080", nil))
	log.Fatal(http.ListenAndServeTLS(":8443", certFile, keyFile, nil))
}

func handler(w http.ResponseWriter, r *http.Request) {
	fmt.Fprintf(w, "Hi from %s", r.URL.Path[1:])
}

func makeCert() (string, string, error) {
	dir := configdir.New("fly", "fly").QueryFolders(configdir.Global)[0]
	certName := "cert.pem"
	keyName := "key.pem"
	certFile := filepath.Join(dir.Path, certName)
	keyFile := filepath.Join(dir.Path, keyName)
	if dir.Exists(certName) && dir.Exists(keyName) {
		return certFile, keyFile, nil
	}
	// Some from https://golang.org/src/crypto/tls/generate_cert.go
	// Some from https://fale.io/blog/2017/06/05/create-a-pki-in-golang/
	serialNumber, err := randomSerialNumber()
	if err != nil {
		return "", "", err
	}
	ca := &x509.Certificate{
		NotAfter:     time.Now().AddDate(1, 0, 0),
		NotBefore:    time.Now(),
		SerialNumber: serialNumber,
		// Subject: pkix.Name{},
	}
	priv, err := rsa.GenerateKey(rand.Reader, 4096)
	if err != nil {
		return "", "", err
	}
	pub := &priv.PublicKey
	cert, err := x509.CreateCertificate(rand.Reader, ca, ca, pub, priv)
	if err != nil {
		return "", "", err
	}
	// Certificate.
	certBytes := pem.EncodeToMemory(&pem.Block{Type: "CERTIFICATE", Bytes: cert})
	err = dir.WriteFile(certName, certBytes)
	if err != nil {
		return "", "", err
	}
	// Key.
	keyOut, err := os.OpenFile(keyFile, os.O_WRONLY|os.O_CREATE|os.O_TRUNC, 0600)
	if err != nil {
		return "", "", err
	}
	defer keyOut.Close()
	pem.Encode(keyOut, &pem.Block{
		Type:  "RSA PRIVATE KEY",
		Bytes: x509.MarshalPKCS1PrivateKey(priv),
	})
	return certFile, keyFile, nil
}

func randomSerialNumber() (*big.Int, error) {
	// From https://golang.org/src/crypto/tls/generate_cert.go
	serialNumberLimit := new(big.Int).Lsh(big.NewInt(1), 128)
	return rand.Int(rand.Reader, serialNumberLimit)
}
