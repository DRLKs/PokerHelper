import java.awt.event.ActionEvent;
import java.awt.event.ActionListener;
import java.beans.PropertyChangeEvent;
import java.beans.PropertyChangeListener;
import java.util.ArrayList;
import java.util.List;

public class Controlador implements ActionListener,PropertyChangeListener {
	
	private Panel panel;
	private Worker worker = null;
	private List<Carta> cartas;
	
	private boolean datosSuficientes = false;
	
	public Controlador(Panel panel)  {
		this.panel = panel;
	}
	
	@Override
	public void actionPerformed(ActionEvent event) {
		String eventCommando = event.getActionCommand();
		
		try {
			if( eventCommando.compareTo("NUEVAPARTIDA") == 0 ) {
				
				panel.limpiarAreasCartas();
				
				if( worker != null) {
					worker.cancel(true);
				}
			}else{
				if( worker != null) {
					worker.cancel(true);
				}
				obtenerCartas();
				
				if( datosSuficientes ) {
					int numContrincantes = panel.numJugadoresActuales();
					panel.setErrores("Calculando Datos");
					
					worker = new Worker(numContrincantes, cartas, panel);
					worker.addPropertyChangeListener(this);
					worker.execute();
				}
			}
		
		
		}catch(NumberFormatException e) {
			panel.setErrores("CONTRINCANTES VACIOS");
		}catch(PokerException e) {	// NO FUNCIONA EL SETERROES, no es el try catch
			panel.setErrores( e.getMessage() );
		}catch( Exception e) {
			e.printStackTrace();
		}
	}
	
	// OBTENEMOS LAS CARTAS QUE USAREMOS EN EL ALGORITMO
	private void obtenerCartas(){
		
		cartas = new ArrayList<>();
		
		for( int i = 1 ; i <= 7 ; ++i ) {
			String cartaString = panel.getEntradas(i);
			if( cartaString != null && cartaString.compareTo("") != 0 ) {
				cartas.add( string_a_carta(cartaString) );
			}
			if( i == 2 && cartas.size() == 2 ) {	// Están las 2 cartas de la mano
				datosSuficientes = true;
			}else if( i == 2 ){
				datosSuficientes = false;
				i = 9;	// Para cortar el bucle
			}
		}
	}
	
	private Carta string_a_carta( String carta ) {
		
		char palo = carta.charAt(0);
		int valor_carta = Integer.parseInt(carta.substring(1));
		
		return new Carta(palo, valor_carta);
	}

	@Override
	public void propertyChange(PropertyChangeEvent evt) {
		// TODO Auto-generated method stub
		
	}
}
